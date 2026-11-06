#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Private, unapproved proof-tool provisioning. Only fetch uses the network.

No candidate is runtime authority. Unknown effects retain their original paths;
this program neither removes candidates nor retries a failed publication.
"""
import argparse
import bz2
import hashlib
import json
import lzma
import os
from pathlib import Path
import platform
import posixpath
import secrets
import signal
import stat
import subprocess
import sys
import tarfile
import time
import urllib.request
import io
import re
import tempfile
import unittest
import zlib

ROOT = Path(__file__).resolve().parent.parent
SCRIPT_BYTES = Path(__file__).read_bytes()
FIX = ROOT / "tests/fixtures/dream-proof-tools"
VERIFIER = "/opt/homebrew/Cellar/gnupg/2.5.24_1/bin/gpg"
VERIFIER_SHA = "21cb9adcad96b7be404014ff6e9fb837160a2135455c217dec9efe196cf0661a"
AUTHORITY = Path("/Users/feral/mydev/bboxy450-term/target/dream-proof-component-authority-7m7jv65m")
PROBE_SHA = "5493dbf0bda41ac33e4a27a9bc200f083e49501b0791d32cd255b731d6f82c33"
Q2_SHA = "55aabc5e0efd081fe050db08a40e955ddb27ff5d8a442df1c6feb911ce8160af"
GIT_SIGNATURE_TEMP = "{git_signature_temp}"
# Root's authenticated source read set for decision65960 B. These are source
# bytes only: no moving Q2 source/probe, GS1/GS2 decoder or runtime authority.
SIGNATURE_SOURCE_RUN = ROOT / "target/dream-proof-tools/runs/01M6Q80DBV75S95DWFHZWDVE2F"
SIGNATURE_SOURCE_PINS = {
    "commit.c": "d02e4488fc271f1421dc7e825fd6d78244f4db1b47716927bc5dd8e7bb6c4583",
    "gpg-interface.c": "191dd45b56d9992f3862f4696693c77019185ae785402686d483823468c53aed",
    "tempfile.c": "a34ef38d1981b2b269436288685a4af3453d102bac2f29fc2449b8c98b6279d2",
    "tempfile.h": "20c6e57636813f056554fdbb3a20652257aac3d93fd7dd367ec7b785f6f53a1a",
    "wrapper.c": "788c07b315a6e98bfcaf78291b83b17e52579f17b604a94db5c149c017a83b04",
    "run-command.c": "03bf7f114875b964cb9397893b3ddceafd4d71f51b908a9c02ae6ca7e2a4d67f",
}
MAX_ARCHIVE = 32 * 1024 * 1024
MAX_TAR = 256 * 1024 * 1024
MAX_INT = 2**53 - 1
KEYS = {
    "git": ("https://github.com/gitster.gpg", "2a3477639ac9bff6c814a679983408c70e5320402a38d41a74b2df0b206e9bfc"),
    "gnupg": ("https://gnupg.org/signature_key.asc", "6f57d0e17fefd2238bd037aebf03efa960641629dd61dd9d47f122a829f6e375"),
}
GIT_ROOT = "96E07AF25771955980DAD10020D04E5A713660A7"
GIT_SUB = "E1F036B1FEE7221FC778ECEFB0B5E88696AFE6CB"
GPG_ROOTS = ["6DAA6E64A76D2840571B4902528897B826403ADA", "AC8E115BF73E2D8D47FA9908E98E9B2D19C6C8BD"]
PINS = [
    ("git", "2.54.0", "https://www.kernel.org/pub/software/scm/git/git-2.54.0.tar.xz", "f689162364c10de79ef89aa8dbf48731eb057e34edbbd20aca510ce0154681a3", "1083f5c8eafe129da815da3f78cbc5865bdaec29be0d3969c104d3b003801775"),
    ("gnupg", "2.5.24", "https://gnupg.org/ftp/gcrypt/gnupg/gnupg-2.5.24.tar.bz2", "bf149d01a2b9fcc0e4589b8ae8697d3d5c557ea48ed95a3fa55dd3b1187e6039", "6cc9dad0a8ad9a27e842c06daf696c94db5c83237933687e773c02b04857fb1a"),
    ("libgpg-error", "1.61", "https://gnupg.org/ftp/gcrypt/libgpg-error/libgpg-error-1.61.tar.bz2", "7a85413f2bc354f4f8aa832b718af122e48965e9e0eb9012ee659c13c6385c93", "9b20f1e6046aaa3fe16fcd65587fd0fa5e34ceb4830f8eadc856c3e80ad57ad3"),
    ("libgcrypt", "1.12.4", "https://gnupg.org/ftp/gcrypt/libgcrypt/libgcrypt-1.12.4.tar.bz2", "d77f68f48879510e79a2f65977ccc68981781ea0923e5bdffac2a193ea3d660e", "8be53f19f43ffa0297aceb520be717de539d1bdad45b5538a57fd95610b1e2db"),
    ("libassuan", "3.0.2", "https://gnupg.org/ftp/gcrypt/libassuan/libassuan-3.0.2.tar.bz2", "d2931cdad266e633510f9970e1a2f346055e351bb19f9b78912475b8074c36f6", "5aa3c5cea6f42bcb96be9e1dd922b33ccb05e2f0565e93674e85ddcf8cd78e86"),
    ("npth", "1.8", "https://gnupg.org/ftp/gcrypt/npth/npth-1.8.tar.bz2", "8bd24b4f23a3065d6e5b26e98aba9ce783ea4fd781069c1b35d149694e90ca3e", "eb9314cf3867a438cb0fdd371125d44c57ba66086f152199ad5f40a9c9679acb"),
    ("libksba", "1.8.1", "https://gnupg.org/ftp/gcrypt/libksba/libksba-1.8.1.tar.bz2", "c2f84393011827219ae117131dba8e7684c2bed0961eed11b0642c2acba440b5", "af99f325f81eb395d740bf48dc8e89903b141c10f69244e46e4caa1caec0577e"),
]


class Refused(Exception):
    pass


def require(ok, reason):
    if not ok:
        raise Refused(reason)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(value):
    def check(node, depth=0):
        require(depth <= 10, "JSON depth bound")
        if isinstance(node, str):
            require("\x00" not in node and not any(0xD800 <= ord(c) <= 0xDFFF for c in node), "JSON NUL/surrogate")
        elif isinstance(node, dict):
            for key, child in node.items():
                require(isinstance(key, str) and key.isascii(), "ASCII JSON keys required")
                check(key, depth + 1); check(child, depth + 1)
        elif isinstance(node, list):
            for child in node: check(child, depth + 1)
        else:
            require(node is None or type(node) in (int, bool), "JSON integers only")
    check(value)
    return json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode()


def private_dir(path):
    path = Path(path)
    if not path.exists():
        path.mkdir(mode=0o700)
    return check_private_dir(path)


def check_private_dir(path):
    path = Path(path)
    s = path.lstat()
    require(stat.S_ISDIR(s.st_mode) and s.st_uid == os.getuid() and stat.S_IMODE(s.st_mode) == 0o700, f"unsafe private directory: {path}")
    return path


def read_regular(path, limit=MAX_TAR, private=False, *, platform_image=False):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        s = os.fstat(fd)
        require(stat.S_ISREG(s.st_mode) and (s.st_nlink == 1 or (platform_image and not private)) and s.st_size <= limit, f"unsafe/oversized regular file: {path}")
        if private:
            require(s.st_uid == os.getuid() and stat.S_IMODE(s.st_mode) in (0o600, 0o700), f"unsafe custody: {path}")
        data = bytearray()
        while len(data) <= limit:
            chunk = os.read(fd, min(65536, limit + 1 - len(data)))
            if not chunk:
                break
            data.extend(chunk)
        t = os.fstat(fd)
        require(len(data) == s.st_size and (s.st_dev, s.st_ino, s.st_mtime_ns, s.st_ctime_ns, s.st_size) == (t.st_dev, t.st_ino, t.st_mtime_ns, t.st_ctime_ns, t.st_size), f"changed read: {path}")
        return bytes(data)
    finally:
        os.close(fd)


def sync_dir(path):
    fd = os.open(path, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW)
    try:
        os.fsync(fd)
    finally:
        os.close(fd)


def exclusive(path, data, executable=False):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW | os.O_CLOEXEC, 0o700 if executable else 0o600)
    try:
        view = memoryview(data)
        while view:
            n = os.write(fd, view)
            require(n > 0, f"unknown partial write retained at {path}")
            view = view[n:]
        os.fsync(fd)
    finally:
        os.close(fd)
    sync_dir(Path(path).parent)


def publish(path, data):
    # A same-directory hard-link is atomic no-replace on Darwin. Do not unlink
    # a temp after an unknown link/fsync outcome; its inode is the effect ID.
    temp = path.with_name(path.name + ".pending-" + secrets.token_hex(8))
    exclusive(temp, data)
    os.link(temp, path, follow_symlinks=False)
    sync_dir(path.parent)
    temp.unlink()
    sync_dir(path.parent)


def ulid():
    n = (time.time_ns() // 1_000_000 << 80) | int.from_bytes(secrets.token_bytes(10), "big")
    alphabet = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
    return "".join(alphabet[(n >> shift) & 31] for shift in range(125, -1, -5))


def env(home):
    return {"PATH": "/usr/bin:/bin:/usr/sbin:/sbin", "LANG": "C", "LC_ALL": "C", "HOME": str(home), "GNUPGHOME": str(home), "TMPDIR": str(home), "GIT_CONFIG_NOSYSTEM": "1", "GIT_CONFIG_GLOBAL": "/dev/null", "GIT_TERMINAL_PROMPT": "0"}


def os_arch():
    # The unchanged DTO uses Rust std::env::consts::ARCH, whose Darwin ARM name
    # is aarch64. Preserve the actual uname machine separately in private logs.
    return {"arm64": "aarch64"}.get(platform.machine(), platform.machine())


def invoke(argv, home, *, data=None, cwd=None, timeout=600):
    result = subprocess.run(argv, input=data, cwd=cwd, env=env(home), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    return result


def gpg(home, args, data=None):
    return invoke([VERIFIER, "--no-options", "--batch", "--no-tty", "--no-autostart", "--no-auto-key-retrieve", "--no-auto-key-import", "--auto-key-locate", "clear", "--no-auto-check-trustdb", *args], home, data=data)


def pinned(data, expected, label):
    require(sha(data) == expected, f"hash mismatch: {label}: actual {sha(data)} expected {expected}")
    return data


def quarantine_cache(cache, run, source, error):
    check_private_dir(cache)
    path = cache / "quarantine.json"
    if not path.exists():
        info = cache.stat()
        exclusive(path, canonical({"source": source[0], "archive_sha256": source[3], "original_run": run.name,
                                  "device": info.st_dev, "inode": info.st_ino, "reason": str(error)[:2048]}))


def cached_source(d, source, run):
    cache = check_private_dir(d / "cache" / source[3])
    require(not (cache / "quarantine.json").exists(), f"exact source cache is quarantined: {cache}; no implicit replacement or root refresh")
    try:
        archive = pinned(read_regular(cache / "archive", MAX_ARCHIVE, True), source[3], "cached source archive")
        signature = pinned(read_regular(cache / "signature", MAX_ARCHIVE, True), source[4], "cached source signature")
        return archive, signature
    except (Refused, OSError) as error:
        quarantine_cache(cache, run, source, error)
        raise


def download(url):
    require(url.startswith("https://"), "HTTPS required")
    # Redirects cannot downgrade TLS and pinned bytes decide acceptance.
    class HTTPSOnly(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, req, fp, code, msg, headers, newurl):
            require(newurl.startswith("https://"), "non-HTTPS redirect")
            return super().redirect_request(req, fp, code, msg, headers, newurl)
    with urllib.request.build_opener(HTTPSOnly).open(url, timeout=45) as response:
        data = response.read(MAX_ARCHIVE + 1)
    require(len(data) <= MAX_ARCHIVE, "download exceeds 32MiB")
    return data


def tar_bytes(archive, compression):
    decoder = lzma.LZMADecompressor() if compression == "xz" else bz2.BZ2Decompressor()
    data = decoder.decompress(archive, max_length=MAX_TAR + 1)
    require(len(data) <= MAX_TAR and decoder.eof and not decoder.unused_data, "archive expansion/trailing data unsupported")
    return data


def authenticate(source, archive, signature, key, home):
    pinned(read_regular(VERIFIER), VERIFIER_SHA, "independent bootstrap verifier")
    pinned(archive, source[3], source[0])
    pinned(signature, source[4], source[0] + " signature")
    pinned(key, KEYS["git" if source[0] == "git" else "gnupg"][1], "public root")
    # Show-only parsing of a checksum-pinned public release certificate. Secret
    # packets are never imported, even if a future pin is accidentally changed.
    packets = gpg(home, ["--list-packets"], key)
    require(packets.returncode == 0 and b"secret key packet" not in packets.stdout and b"secret sub key packet" not in packets.stdout, "public release certificate required")
    # import-export keeps this public-only conversion in memory. Ordinary
    # --import consults gpg-agent even for public certificates; never start it
    # or suppress that error. The converted legacy keyring is an owned file.
    imported = gpg(home, ["--import-options", "import-export", "--import"], key)
    exclusive(home / (source[0] + ".import.status"), imported.stdout + b"\nSTDERR\n" + imported.stderr)
    require(imported.returncode == 0 and imported.stdout, f"public release key import-export failed: {imported.returncode} {imported.stderr!r}")
    keyring = home / (source[0] + ".public.gpg")
    exclusive(keyring, imported.stdout)
    sigpath = home / (source[0] + ".sig")
    exclusive(sigpath, signature)
    signed = tar_bytes(archive, "xz") if source[0] == "git" else archive
    result = gpg(home, ["--no-default-keyring", "--keyring", str(keyring), "--status-fd=1", "--verify", str(sigpath), "-"], signed)
    raw = result.stdout + b"\nSTDERR\n" + result.stderr
    exclusive(home / (source[0] + ".status"), raw)
    require(result.returncode == 0, f"source authentication failed: {source[0]} exit={result.returncode} status={raw.decode(errors='replace')}")
    bad = (b"BADSIG", b"ERRSIG", b"REVKEYSIG", b"EXPKEYSIG", b"EXPSIG", b"KEYREVOKED", b"KEYEXPIRED", b"SIGEXPIRED")
    require(not any(b"[GNUPG:] " + token in raw for token in bad), "expired/revoked/invalid source signature")
    valid = [line.split() for line in result.stdout.decode().splitlines() if line.startswith("[GNUPG:] VALIDSIG ")]
    expected = [GIT_ROOT] if source[0] == "git" else GPG_ROOTS
    require(valid and all(len(v) >= 11 and v[2] in ([GIT_SUB] if source[0] == "git" else GPG_ROOTS) and v[-1] in expected and 0 < int(v[4]) <= int(time.time()) and int(v[5]) in (0,) for v in valid), "unbound signer/subkey/time")
    return {"name": source[0], "archive_sha256": source[3], "signature_sha256": source[4], "signer_primary": valid[0][-1], "signer_subkey": valid[0][2], "verifier_sha256": VERIFIER_SHA, "auth_exit": result.returncode, "auth_status_sha256": sha(raw)}, raw


def session(d, kind):
    build_id = ulid()
    run = private_dir(d / "runs" / build_id)
    # Milliseconds stay exact in the declared JSON integer domain. The monotonic
    # incarnation is retained with the original run, never reset on a refusal.
    previous = []
    for path in (d / "runs").glob("*/identity.json"):
        record = strict_json(read_regular(path, 32768, True), 32768)
        checked_int(record["incarnation"], 1)
        previous.append(record["incarnation"])
    incarnation = max(time.monotonic_ns(), max(previous, default=0) + 1)
    checked_int(incarnation, 1)
    exclusive(run / "identity.json", canonical({"build_id": build_id, "incarnation": incarnation, "kind": kind, "pid": os.getpid(), "created_ms": time.time_ns() // 1_000_000}))
    exclusive(run / "recipe.py", SCRIPT_BYTES)
    return run


def fetch(d):
    run = session(d, "fetch")
    pinned(read_regular(VERIFIER), VERIFIER_SHA, "independent bootstrap verifier")
    home = private_dir(run / "public-home")
    keys = {}
    for name, (url, digest) in KEYS.items():
        path = FIX / (name + "-release-key.asc")
        if path.exists():
            keys[name] = pinned(read_regular(path, MAX_ARCHIVE), digest, name + " key")
        else:
            keys[name] = pinned(download(url), digest, name + " key")
            exclusive(path, keys[name])
    for source in PINS:
        name, _, url, digest, sigsha = source
        cache = private_dir(d / "cache" / digest)
        require(not (cache / "quarantine.json").exists(), f"source cache quarantined: {cache}")
        archive = pinned(read_regular(cache / "archive", MAX_ARCHIVE, True), digest, name) if (cache / "archive").exists() else pinned(download(url), digest, name)
        sigurl = url[:-3] + ".sign" if name == "git" else url + ".sig"
        sigfile = FIX / (name + ".sig")
        signature = pinned(read_regular(sigfile, MAX_ARCHIVE), sigsha, name + " signature") if sigfile.exists() else pinned(download(sigurl), sigsha, name + " signature")
        if not sigfile.exists():
            exclusive(sigfile, signature)
        # A cached auth.json never substitutes for fresh crypto verification.
        try:
            record, raw = authenticate(source, archive, signature, keys["git" if name == "git" else "gnupg"], home)
        except (Refused, OSError) as error:
            quarantine_cache(cache, run, source, error)
            raise
        for basename, data in [("archive", archive), ("signature", signature), ("auth.json", canonical(record)), ("auth.status", raw)]:
            path = cache / basename
            if path.exists():
                old = read_regular(path, MAX_TAR, True)
                # Status output may contain harmless fresh import diagnostics;
                # retained original status remains the cache's audit record.
                if basename in ("archive", "signature"):
                    require(old == data, f"cache conflict retained: {path}")
            else:
                publish(path, data)
        print(f"authenticated {name} {digest} signer={record['signer_primary']}", flush=True)
    locked = canonical(source_lock())
    path = FIX / "sources.lock.json"
    if path.exists():
        require(read_regular(path, 32768) == locked, "source lock changed; explicit reviewed re-pin required")
    else:
        exclusive(path, locked)


def extract(archive, compression, parent, expected):
    raw = tar_bytes(archive, compression)
    with tarfile.open(fileobj=io.BytesIO(raw), mode="r:") as tar:
        members = tar.getmembers()
        require(len(members) <= 40000, "archive entry bound")
        total = 0
        seen = set()
        for entry in members:
            parts = Path(entry.name).parts
            require(parts and parts[0] == expected and not Path(entry.name).is_absolute() and all(p not in (".", "..") for p in parts), "unsafe archive path")
            require(entry.isdir() or entry.isfile() or entry.issym(), f"archive hardlink/special entry unsupported: {entry.name}")
            if entry.issym():
                target = posixpath.normpath(posixpath.join(posixpath.dirname(entry.name), entry.linkname))
                require(not entry.linkname.startswith("/") and target.startswith(expected + "/"), "escaping archive symlink")
            require(entry.name not in seen, "duplicate archive entry")
            seen.add(entry.name)
            total += entry.size
            require(total <= MAX_TAR and entry.size <= MAX_ARCHIVE, "extraction bound")
        # Validate the entire table before the first filesystem effect.
        for entry in members:
            dest = parent.joinpath(*Path(entry.name).parts)
            if entry.issym():
                continue
            if entry.isdir():
                if not dest.exists():
                    dest.mkdir(mode=0o700, parents=True)
                private_dir(dest)
            else:
                dest.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                with tar.extractfile(entry) as stream:
                    exclusive(dest, stream.read(entry.size + 1), bool(entry.mode & 0o111))
        # Links are created only after regular writes, cannot have served as a
        # write ancestor, and must resolve directly to an extracted regular file.
        for entry in members:
            if entry.issym():
                target = parent / posixpath.normpath(posixpath.join(posixpath.dirname(entry.name), entry.linkname))
                require((target.is_file() or target.is_dir()) and not target.is_symlink(), f"unresolved/indirect archive link: {entry.name}")
                os.symlink(entry.linkname, parent / entry.name)


def build(d):
    refuse_unknown_build_effects(d)
    run = session(d, "build")
    home = private_dir(run / "public-home")
    src = private_dir(run / "src")
    private_dir(run / "build")
    private_dir(run / "logs")
    validate_lock(read_regular(FIX / "sources.lock.json", 32768))
    exclusive(run / "sources.lock.json", read_regular(FIX / "sources.lock.json", 32768))
    inputs = build_inputs(run)
    cached = previous_build(d, inputs, run.name)
    if cached is not None:
        # Full artifact/currentness/authentication validation is mandatory even
        # for this builder's own winner; a journal marker never admits bytes.
        return reuse_candidate(d, cached, run, inputs)
    candidate = private_dir(d / "candidates" / run.name)
    identity = strict_json(read_regular(run / "identity.json", 32768, True), 32768)
    info = candidate.stat()
    exclusive(run / "intent.json", canonical({"build_id": run.name, "incarnation": identity["incarnation"], "inputs": inputs,
              "prefix": str(candidate), "device": info.st_dev, "inode": info.st_ino}))
    try:
        return build_new(d, run, candidate)
    except Exception as error:
        # Do not manufacture absent effects after an unknown spawn/write. All
        # original start/PID/inode records remain; same-input retry is refused.
        if not (run / "terminal.json").exists():
            exclusive(run / "terminal.json", canonical({"outcome": "unsupported", "reason": str(error)[:2048], "effect_unknown": bool(pending_commands(run))}))
        raise


def build_new(d, run, candidate):
    prepare_sources(d, run)
    print(f"authenticated sources extracted: {run}; candidate prefix: {candidate}", flush=True)
    return compile_sources(run, candidate)


def prepare_sources(d, run):
    home = run / "public-home"
    src = run / "src"
    records = []
    pinned(read_regular(VERIFIER), VERIFIER_SHA, "independent bootstrap verifier")
    for source in PINS:
        cache = d / "cache" / source[3]
        archive, signature = cached_source(d, source, run)
        key = read_regular(FIX / (("git" if source[0] == "git" else "gnupg") + "-release-key.asc"), MAX_ARCHIVE)
        try:
            record, raw = authenticate(source, archive, signature, key, home)
        except (Refused, OSError) as error:
            quarantine_cache(cache, run, source, error)
            raise
        records.append(record)
        extract(archive, "xz" if source[0] == "git" else "bz2", src, source[0] + "-" + source[1])
    exclusive(run / "authenticated.json", canonical(records))
    apply_profile(run)
    snapshot = expected_sources(d, src)
    verify_source_snapshot(src, snapshot)
    exclusive(run / "source-snapshot.json", canonical(snapshot))
    return snapshot


def check_sources(d):
    run = session(d, "source-custody")
    private_dir(run / "public-home"); private_dir(run / "src"); private_dir(run / "logs")
    snapshot = prepare_sources(d, run)
    # Direct entry mutants, rather than appended/unreachable marker text. Each
    # changed actual Git main is compiled to an object and rejected by SOURCE
    # custody. Objects are never linked/executed; no hidden effect is launched.
    # This does not replace Q2's canonical before-birth/process witnesses or
    # establish that a census would observe the corresponding hidden event.
    _, cc, _ = command(run, ["/usr/bin/xcrun", "--find", "clang"])
    _, sdk, _ = command(run, ["/usr/bin/xcrun", "--show-sdk-path"])
    path = run / "src/git-2.54.0/common-main.c"
    original = read_regular(path, MAX_ARCHIVE, True)
    include = b'#include "common-init.h"\n'
    entry = b"\tint result;\n"
    require(original.count(include) == 1 and original.count(entry) == 1, "Git main entry changed")
    mutations = [("spawn", b"", b"\t(void)fork();\n", b"_fork"),
                 ("thread", b"#include <pthread.h>\nstatic void *dream_hidden_thread(void *p) { return p; }\n", b"\t{ pthread_t t; (void)pthread_create(&t, NULL, dream_hidden_thread, NULL); }\n", b"_pthread_create"),
                 ("service", b"#include <sys/socket.h>\n#include <sys/un.h>\n", b'\t{ struct sockaddr_un a = {.sun_family = AF_UNIX, .sun_path = "dream-private-service-mutant"}; int fd = socket(AF_UNIX, SOCK_STREAM, 0); (void)connect(fd, (struct sockaddr *)&a, sizeof a); }\n', b"_connect")]
    outcomes = []
    for name, headers, code, symbol in mutations:
        mutated = original.replace(include, include + headers).replace(entry, entry + code)
        exclusive(run / "logs" / (name + ".source-input"), mutated)
        path.write_bytes(mutated)
        refused = False
        try:
            verify_source_snapshot(run / "src", snapshot)
        except Refused:
            refused = True
        finally:
            path.write_bytes(original)
        require(refused, f"changed source input retained qualification: {name}")
        source = run / "logs" / (name + ".c")
        obj = run / "logs" / (name + ".o")
        exclusive(source, mutated)
        command(run, [cc.decode().strip(), "-O0", "-DNO_OPENSSL", "-DNO_CURL", "-DNO_GETTEXT", "-DNO_PTHREADS", "-isysroot", sdk.decode().strip(), "-I" + str(path.parent), "-c", str(source), "-o", str(obj)])
        _, undefined, _ = command(run, ["/usr/bin/nm", "-u", str(obj)])
        require(symbol in undefined, "compiler removed the direct entry mutant call")
        outcomes.append({"kind": name, "source_sha256": sha(mutated), "object_sha256": sha(read_regular(obj, MAX_ARCHIVE, True)), "entry_call": symbol.decode(), "SOURCE_qualified": False, "executed": False})
    verify_source_snapshot(run / "src", snapshot)
    exclusive(run / "source-custody.json", canonical({"source_snapshot_sha256": sha(canonical(snapshot)), "compiled_entry_mutants": outcomes, "runtime_lifetime": "unproven"}))
    print(f"authenticated source/patch custody checked at {run}; native lifetime qualification remains unproven", flush=True)


def expected_sources(d, src):
    expected = {}
    for name, version, _, digest, _ in PINS:
        data = pinned(read_regular(d / "cache" / digest / "archive", MAX_ARCHIVE, True), digest, "source snapshot archive")
        with tarfile.open(fileobj=io.BytesIO(tar_bytes(data, "xz" if name == "git" else "bz2")), mode="r:") as tar:
            members = tar.getmembers()
            require(len(members) <= 40000, "source snapshot entry bound")
            for entry in members:
                path = "/".join(Path(entry.name).parts)
                require(path and not Path(path).is_absolute() and all(p not in ("", ".", "..") for p in path.split("/")) and path not in expected, "source snapshot path/duplicate")
                if entry.isfile():
                    with tar.extractfile(entry) as stream:
                        contents = stream.read(MAX_ARCHIVE + 1)
                    require(len(contents) == entry.size <= MAX_ARCHIVE, "source snapshot member bound")
                    expected[path] = {"kind": "file", "sha256": sha(contents), "mode": 0o700 if entry.mode & 0o111 else 0o600}
                elif entry.isdir():
                    expected[path] = {"kind": "directory", "mode": 0o700}
                else:
                    require(entry.issym(), "source snapshot special file")
                    expected[path] = {"kind": "symlink", "target": entry.linkname}
    for patch in strict_json(read_regular(FIX / "patches.lock.json", 32768), 32768):
        for file in patch["files"]:
            key = patch["source"] + "/" + file["path"]
            require(expected[key]["sha256"] == file["before"], "patch/source snapshot preimage differs")
            expected[key]["sha256"] = file["after"]
    return expected


def verify_source_snapshot(src, expected):
    # Whole-table equality catches a new constructor/helper or an alteration in
    # any unpatched upstream file. It is one input-custody check, not a theorem
    # about compiler fidelity, selected reachability or arbitrary native code.
    observed = {}
    stack = [Path(src)]
    while stack:
        parent = stack.pop()
        check_private_dir(parent)
        for entry in os.scandir(parent):
            path = Path(entry.path)
            key = path.relative_to(src).as_posix()
            info = entry.stat(follow_symlinks=False)
            if stat.S_ISLNK(info.st_mode):
                observed[key] = {"kind": "symlink", "target": os.readlink(path)}
            elif stat.S_ISDIR(info.st_mode):
                observed[key] = {"kind": "directory", "mode": stat.S_IMODE(info.st_mode)}
                stack.append(path)
            else:
                require(stat.S_ISREG(info.st_mode), "source special file changed")
                observed[key] = {"kind": "file", "sha256": sha(read_regular(path, MAX_ARCHIVE, True)), "mode": stat.S_IMODE(info.st_mode)}
    require(observed == expected, "SOURCE qualification lost: extracted/guarded source bytes, membership or custody changed")


def pending_commands(run):
    # The start record precedes native process creation. Lack of an exit is
    # unknown, including a crash between Popen and publishing its exact PID.
    return [path for path in (run / "logs").glob("*.command.json")
            if not path.with_name(path.name.replace(".command.json", ".exit.json")).exists()]


def refuse_unknown_build_effects(d):
    for run in (d / "runs").iterdir():
        path = run / "identity.json"
        if path.exists() and strict_json(read_regular(path, 32768, True), 32768).get("kind") == "build":
            private_dir(run)
            pending = pending_commands(run)
            require(not pending, f"original build effect unresolved at {run}; retain its ID/PID/prefix/inode, no replacement retry")


def build_inputs(run):
    _, cc, _ = command(run, ["/usr/bin/xcrun", "--find", "clang"])
    _, sdkout, _ = command(run, ["/usr/bin/xcrun", "--show-sdk-path"])
    _, sdkversion, _ = command(run, ["/usr/bin/xcrun", "--show-sdk-version"])
    _, osbuild, _ = command(run, ["/usr/bin/sw_vers", "-buildVersion"])
    _, commit, _ = command(run, ["/usr/bin/git", "--no-pager", "-c", "core.fsmonitor=false", "-C", str(ROOT), "rev-parse", "HEAD"])
    # Private generated evidence is not source. Build source/patch/recipe changes
    # must have a concrete commit before a review proposal can bind them.
    command(run, ["/usr/bin/git", "--no-pager", "-c", "core.fsmonitor=false", "-C", str(ROOT), "diff", "--exit-code", "HEAD", "--", "Makefile", "scripts/dream-proof-tools.py", "tests/fixtures/dream-proof-tools", "tests/dream_proof_tools.rs", "tests/formal/dream_proof_tools.rs", "tests/formal/models.rs", "verification/corpus.toml", "verification/contracts.toml", "verification/coverage.toml"])
    _, untracked, _ = command(run, ["/usr/bin/git", "--no-pager", "-C", str(ROOT), "ls-files", "--others", "--exclude-standard", "--", "scripts/dream-proof-tools.py", "tests/fixtures/dream-proof-tools", "tests/dream_proof_tools.rs", "tests/formal/dream_proof_tools.rs"])
    require(not untracked, "uncommitted builder source/fixture inputs cannot bind a proposal")
    compiler = cc.decode().strip()
    sdk = sdkout.decode().strip()
    fixtures = {str(p.relative_to(FIX)): sha(read_regular(p, MAX_TAR)) for p in sorted(FIX.rglob("*")) if p.is_file()}
    # The source identifier is the retained recipe FileRef's exact byte hash;
    # the separate complete fixture tuple also participates in cache equality.
    source = sha(SCRIPT_BYTES)
    commit = commit.decode().strip()
    hex_string(commit, 40)
    return {"commit": commit, "source_sha256": source, "fixtures": fixtures, "recipe_sha256": sha(SCRIPT_BYTES),
            "compiler": compiler, "compiler_sha256": sha(read_regular(Path(compiler).resolve())),
            "sdk": sdk, "sdk_version": sdkversion.decode().strip(), "sdk_settings_sha256": sha(read_regular(Path(sdk) / "SDKSettings.json")),
            "os_build": osbuild.decode().strip(), "release": platform.release(), "arch": os_arch()}


def previous_build(d, inputs, current_id):
    winner = None
    for run in sorted((d / "runs").iterdir()):
        if run.name == current_id or not (run / "intent.json").exists():
            continue
        private_dir(run)
        intent = strict_json(read_regular(run / "intent.json", 32768, True), 32768)
        if intent["inputs"] != inputs:
            continue
        prefix = Path(intent["prefix"])
        require(prefix == d / "candidates" / run.name, "original candidate path mismatch")
        check_private_dir(prefix)
        info = prefix.stat()
        require((info.st_dev, info.st_ino) == (intent["device"], intent["inode"]), f"candidate incarnation/ABA changed at {prefix}")
        if (prefix / "receipt.candidate.json").exists():
            require(winner is None, "multiple same-input candidate effects require inspection, not winner guessing")
            winner = run
            continue
        if (run / "terminal.json").exists():
            terminal = strict_json(read_regular(run / "terminal.json", 32768, True), 32768)
            raise Refused(f"retained original build {run.name} outcome={terminal['outcome']}: {terminal['reason']}; unchanged inputs do not authorize another attempt")
        raise Refused(f"incomplete original build {run.name} retains its exact effect/prefix/inode; no blind restart or cleanup")
    return winner


def command(run, argv, *, cwd=None, extra_env=None, allow_failure=False):
    logs = run / "logs"
    seq = len(list(logs.glob("*.command.json")))
    stem = logs / f"{seq:03d}"
    effective = env(run / "public-home")
    effective.update(extra_env or {})
    request = {"argv": argv, "cwd": str(cwd or run), "env": effective, "source_sha256": sha(SCRIPT_BYTES)}
    require(len(argv) <= 64 and sum(len(s.encode()) + 1 for s in argv) + sum(len(k.encode()) + len(v.encode()) + 2 for k, v in effective.items()) <= 32768, "command argv/environment byte bound")
    exclusive(stem.with_suffix(".command.json"), canonical(request))
    # Every build is serial at this owner; make itself gets exactly two niced
    # jobs. A lost process outcome leaves this start record without an exit.
    with open(stem.with_suffix(".stdout"), "xb", opener=lambda p, flags: os.open(p, flags, 0o600)) as out, open(stem.with_suffix(".stderr"), "xb", opener=lambda p, flags: os.open(p, flags, 0o600)) as err:
        exclusive(stem.with_suffix(".spawn-intent.json"), canonical({"operation": stem.name, "build_id": run.name, "effect": "process_creation_pending"}))
        child = subprocess.Popen(["/usr/bin/nice", "-n", "10", *argv], cwd=cwd or run, env=effective, stdout=out, stderr=err, start_new_session=True)
        exclusive(stem.with_suffix(".pid.json"), canonical({"pid": child.pid, "pgid": child.pid, "operation": stem.name, "build_id": run.name}))
        try:
            status = child.wait(timeout=1200)
        except subprocess.TimeoutExpired:
            # This unreaped leader still pins the exact new session/group we
            # created. Signal only that group, before wait/reap; never signal a
            # numeric PGID afterward or claim escaped-descendant coverage.
            for sig in (signal.SIGTERM, signal.SIGKILL):
                try:
                    os.killpg(child.pid, sig)
                except ProcessLookupError:
                    pass
            child.wait()
            exclusive(stem.with_suffix(".timeout.json"), canonical({"pid": child.pid, "exit_code": child.returncode, "descendant_coverage": "unknown"}))
            raise Refused(f"build command timeout; original command/PID/effect retained: {stem}")
        out.flush(); os.fsync(out.fileno())
        err.flush(); os.fsync(err.fileno())
    stdout = read_regular(stem.with_suffix(".stdout"))
    stderr = read_regular(stem.with_suffix(".stderr"))
    exclusive(stem.with_suffix(".exit.json"), canonical({"exit_code": status, "stdout_sha256": sha(stdout), "stderr_sha256": sha(stderr)}))
    print(f"command {stem.name} exit={status}: {' '.join(argv[:4])}", flush=True)
    require(status == 0 or allow_failure, f"command failed exit={status}; retained {stem}.stdout/.stderr")
    return status, stdout, stderr


def compile_sources(run, candidate):
    _, cc, _ = command(run, ["/usr/bin/xcrun", "--find", "clang"])
    compiler = cc.decode().strip()
    _, sdkout, _ = command(run, ["/usr/bin/xcrun", "--show-sdk-path"])
    sdk = sdkout.decode().strip()
    command(run, [compiler, "--version"])
    _, ldout, _ = command(run, ["/usr/bin/xcrun", "--find", "ld"])
    command(run, ["/usr/bin/xcrun", "--show-sdk-version"])
    command(run, ["/usr/bin/sw_vers", "-buildVersion"])
    linker = ldout.decode().strip()
    exclusive(run / "platform.json", canonical({"compiler": compiler, "compiler_sha256": sha(read_regular(Path(compiler).resolve())), "linker": linker, "linker_sha256": sha(read_regular(Path(linker).resolve())), "sdk": sdk, "sdk_settings_sha256": sha(read_regular(Path(sdk) / "SDKSettings.json")), "os_release": platform.release(), "arch": os_arch(), "machine": platform.machine(), "trust_assumptions": ["independent bootstrap verifier", "compiler/linker and SDK fidelity", "native platform/libSystem implementation; no arbitrary native theorem"]}))
    tools = {}
    for name in ("make", "patch", "otool", "ar", "ranlib", "nice", "sh"):
        path = Path("/bin/sh") if name == "sh" else Path("/usr/bin") / name
        tools[name] = {"path": str(path), "resolved_path": str(path.resolve()), "sha256": sha(read_regular(path.resolve(), platform_image=True)), "links": path.stat().st_nlink}
    tools["python"] = {"path": sys.executable, "resolved_path": str(Path(sys.executable).resolve()), "sha256": sha(read_regular(Path(sys.executable).resolve(), platform_image=True)), "links": Path(sys.executable).stat().st_nlink}
    exclusive(run / "developer-tools.json", canonical(tools))
    deps = private_dir(run / "build" / "deps")
    buildenv = {"CC": compiler, "SDKROOT": sdk, "CFLAGS_FOR_BUILD": "-O2 -isysroot " + sdk, "CFLAGS": "-O2 -g0 -isysroot " + sdk, "CPPFLAGS": "-isysroot " + sdk + " -I" + str(deps / "include"), "LDFLAGS": "-L" + str(deps / "lib") + " -isysroot " + sdk,
                "PATH": str(deps / "bin") + ":/usr/bin:/bin:/usr/sbin:/sbin", "PKG_CONFIG": "/usr/bin/false", "PKG_CONFIG_PATH": str(deps / "lib/pkgconfig"), "PKG_CONFIG_LIBDIR": str(deps / "lib/pkgconfig")}
    order = [PINS[2], PINS[3], PINS[4], PINS[5], PINS[6]]
    for source in order:
        name, version = source[:2]
        work = private_dir(run / "build" / name)
        configure = run / "src" / f"{name}-{version}" / "configure"
        _, helpout, _ = command(run, [str(configure), "--help"], cwd=work, extra_env=buildenv)
        flags = ["--prefix=" + str(deps), "--disable-shared", "--enable-static"]
        if name != "npth":
            flags += ["--disable-doc"]
        if name == "libgpg-error":
            flags += ["--disable-tests", "--disable-languages"]
        elif name in ("libgcrypt", "libassuan", "libksba"):
            flags += ["--with-libgpg-error-prefix=" + str(deps)]
        if name == "libgcrypt":
            # Xcode 27's assembler rejects this release's aarch64 frame
            # directives. Select upstream's portable implementation explicitly;
            # retain all algorithms and real signature semantics.
            flags += ["--disable-jent-support", "--disable-asm"]
        for flag in flags[1:]:
            require(supported_option(flag, helpout), f"unrecognized configure option: {name} {flag}")
        _, out, err = command(run, [str(configure), *flags], cwd=work, extra_env=buildenv)
        require(b"unrecognized option" not in out + err, "configure ignored an option")
        command(run, ["/usr/bin/make", "-j2"], cwd=work, extra_env=buildenv)
        command(run, ["/usr/bin/make", "-j2", "install"], cwd=work, extra_env=buildenv)
        harden_owned_tree(deps)
    build_git(run, candidate, compiler, sdk, buildenv)
    source = run / "src/gnupg-2.5.24"
    work = private_dir(run / "build/gnupg")
    _, helpout, _ = command(run, [str(source / "configure"), "--help"], cwd=work, extra_env=buildenv)
    flags = ["--prefix=" + str(candidate), "--disable-gpgsm", "--disable-scdaemon", "--disable-dirmngr", "--disable-keyboxd", "--disable-tpm2d", "--disable-g13", "--disable-doc", "--disable-gpgtar", "--disable-wks-tools", "--disable-tests", "--disable-exec", "--disable-photo-viewers", "--disable-card-support", "--disable-ccid-driver", "--disable-dirmngr-auto-start", "--disable-sqlite", "--disable-tofu", "--disable-ldap", "--disable-gnutls", "--disable-ntbtls", "--disable-libdns", "--disable-nls", "--without-readline", "--with-libgpg-error-prefix=" + str(deps), "--with-libgcrypt-prefix=" + str(deps), "--with-libassuan-prefix=" + str(deps), "--with-npth-prefix=" + str(deps), "--with-libksba-prefix=" + str(deps)]
    for flag in flags[1:]:
        require(supported_option(flag, helpout), f"unrecognized GnuPG configure option: {flag}")
    status, out, err = command(run, [str(source / "configure"), *flags], cwd=work, extra_env=buildenv, allow_failure=True)
    require(b"unrecognized option" not in out + err, "GnuPG configure ignored an option")
    if status:
        exclusive(run / "unsupported.json", canonical({"reason": "GnuPG configure failed; no dependency substitution authorized", "exit_code": status, "candidate": str(candidate), "build_id": run.name}))
        raise Refused(f"GnuPG configure exit={status}; inspect original logs and generated config in {run}; no additional source/library authorized")
    command(run, ["/usr/bin/make", "-j2", "gpg_LDFLAGS=-Wl,-map," + str(run / "logs/gpg.link-map")], cwd=work, extra_env=buildenv)
    generated = read_regular(work / "config.h", MAX_ARCHIVE, True)
    for macro in ("ENABLE_CARD_SUPPORT", "HAVE_LIBREADLINE", "USE_LDAP", "USE_TOFU"):
        require(("/* #undef " + macro + " */").encode() in generated and not re.search(rb"^#define " + macro.encode() + rb"\b", generated, re.MULTILINE), "unexpected generated GPG feature: " + macro)
    gcrypt = read_regular(run / "build/libgcrypt/config.h", MAX_ARCHIVE, True)
    for macro in ("ENABLE_HMAC_BINARY_CHECK", "ENABLE_JENT_SUPPORT"):
        require(("/* #undef " + macro + " */").encode() in gcrypt, "unexpected generated libgcrypt constructor/entropy feature")
    bindir = private_dir(candidate / "bin")
    exclusive(bindir / "gpg", read_regular(work / "g10/gpg"), executable=True)
    command(run, ["/usr/bin/otool", "-L", str(bindir / "gpg")])
    command(run, ["/usr/bin/otool", "-l", str(bindir / "gpg")])
    stage_linked_archives(run, candidate, deps)
    closure = emit_closure(run, candidate)
    prepare_basis(run, candidate, closure)
    exclusive(run / "build-complete.json", canonical({"candidate": str(candidate), "tools": {name: sha(read_regular(bindir / name, private=True)) for name in ("git", "gpg")}, "lifetime": "unproven_pending_canonical_private_probe"}))
    raise Refused(f"real pair built at {candidate}; exact canonical private-candidate probe freeze still required before joined qualification")


def build_git(run, candidate, compiler, sdk, buildenv):
    # Git's release tarball is already generated, so no autotools or external
    # package discovery is used. Real supported read/signature paths remain.
    work = run / "src/git-2.54.0"
    cflags = "-O2 -g0 -isysroot " + sdk + " -DDREAM_PROOF_GPG='\"" + str(candidate / "bin/gpg") + "\"'"
    # config.mak.uname already disables the Darwin fsmonitor backend under
    # NO_PTHREADS; its unconditional CoreServices link is unnecessary for these
    # forms. The upstream make variable removes that unused framework, with
    # real link/load evidence deciding the resulting closure.
    variables = ["CC=" + compiler, "CFLAGS=" + cflags, "LDFLAGS=-isysroot " + sdk + " -Wl,-map," + str(run / "logs/git.link-map"), "BASIC_LDFLAGS=", "NO_PTHREADS=YesPlease", "NO_CURL=YesPlease", "NO_EXPAT=YesPlease", "NO_OPENSSL=YesPlease", "NO_GETTEXT=YesPlease", "NO_TCLTK=YesPlease", "NO_PERL=YesPlease", "NO_PYTHON=YesPlease", "USE_LIBPCRE=", "NO_FINK=YesPlease", "NO_DARWIN_PORTS=YesPlease", "NO_APPLE_COMMON_CRYPTO=YesPlease", "NO_HOMEBREW=YesPlease", "prefix=" + str(candidate)]
    command(run, ["/usr/bin/make", "-j2", *variables, "git"], cwd=work, extra_env=buildenv)
    for filename in ("GIT-CFLAGS", "GIT-LDFLAGS"):
        observed = read_regular(work / filename, MAX_ARCHIVE, True)
        require(b"/opt/homebrew" not in observed and b"/usr/local" not in observed and b"CoreServices" not in observed, "ambient Homebrew/framework discovery in actual Git flags")
    bindir = private_dir(candidate / "bin")
    exclusive(bindir / "git", read_regular(work / "git"), executable=True)
    command(run, ["/usr/bin/otool", "-L", str(bindir / "git")])
    command(run, ["/usr/bin/otool", "-l", str(bindir / "git")])
    for name, _, *_ in PINS:
        version = next(p[1] for p in PINS if p[0] == name)
        licensedir = private_dir(candidate / "licenses")
        part = private_dir(licensedir / name)
        source = run / "src" / f"{name}-{version}"
        notices = sorted(source.glob("COPYING*")) + ([source / "AUTHORS"] if (source / "AUTHORS").is_file() else [])
        for path in notices:
            exclusive(part / path.name, read_regular(path))
    # Version/image output is provenance only, not a lifetime qualification.
    command(run, [str(bindir / "git"), "--version"])


def stage_linked_archives(run, candidate, deps):
    # Actual Darwin link maps decide runtime/static membership, including KSBA;
    # configure-required does not imply that a library was linked into a tool.
    archives = {}
    selected = {}
    for name in ("git", "gpg"):
        raw = read_regular(run / "logs" / (name + ".link-map"), 8 * 1024 * 1024, True)
        cwd = run / ("src/git-2.54.0" if name == "git" else "build/gnupg/g10")
        for entry in parse_link_map_objects(raw, name):
            match = re.fullmatch(rb"(.+\.a)\(([^()]*)\)", entry)
            if not match:
                continue
            path = Path(link_map_path(match[1]))
            if not path.is_absolute():
                path = cwd / path
            path = path.resolve()
            require(path.is_relative_to(run.resolve()), f"unapproved static archive path: {path}")
            archives[str(path)] = sha(read_regular(path, private=True))
            selected.setdefault(str(path), {}).setdefault(name, []).append(link_map_path(match[2]))
    libdir = private_dir(candidate / "lib")
    summary = []
    for index, (name, digest) in enumerate(sorted(archives.items())):
        path = Path(name)
        staged_name = f"{index:02d}-" + path.name
        closure_retain_exact(libdir / staged_name, pinned(read_regular(path, private=True), digest, "linked archive"))
        status, members, _ = command(run, ["/usr/bin/ar", "-t", name])
        require(len(members) <= 1024 * 1024 and b"\x00" not in members, "archive member-index bound")
        full_index = [link_map_path(member) for member in members.split(b"\n") if member]
        require(all(member in full_index for used in selected[name].values() for member in used), "link map selects a member absent from the real archive index")
        summary.append({"original_path": name, "staged": "lib/" + staged_name, "sha256": digest, "member_index_sha256": sha(members), "member_index": retain_blob(candidate, members), "selected_members": selected[name], "index_exit": status})
    closure_retain_exact(run / "static-closure.json", canonical(summary))


def link_map_path(raw):
    # Only paths/member names in the object table are text. Darwin link maps
    # contain arbitrary literal-string bytes later in the symbol table; do not
    # decode, replace, normalize or discard those bytes to obtain a text map.
    require(0 < len(raw) <= 4096 and b"\x00" not in raw and b"\r" not in raw, "link-map path bound")
    try:
        return raw.decode("utf-8", errors="strict")
    except UnicodeError as error:
        raise Refused("non-UTF-8 object-table path is unsupported; original map retained") from error


def parse_link_map_objects(raw, tool):
    require(tool in ("git", "gpg") and len(raw) <= 8 * 1024 * 1024, "link-map input bound")
    header = b"# Path: " + tool.encode() + b"\n# Arch: arm64\n# Object files:\n"
    require(raw.startswith(header), "unexpected Darwin link-map header/architecture")
    table, separator, _ = raw[len(header):].partition(b"# Sections:\n")
    require(separator and len(table) <= 2 * 1024 * 1024, "missing/oversized link-map object table")
    rows = table.split(b"\n")
    require(rows[-1] == b"" and 1 < len(rows) <= 8193, "link-map object-row bound")
    result = []
    for index, line in enumerate(rows[:-1]):
        match = re.fullmatch(rb"\[\s*([0-9]+)\]\s+([^\x00\r\n]+)", line)
        require(match is not None and int(match[1]) == index, "malformed/duplicate/noncontiguous link-map object index")
        require(len(match[2]) <= 4096, "link-map object path bound")
        if index == 0:
            require(match[2] == b"linker synthesized", "unexpected synthesized object entry")
        else:
            result.append(match[2])
    return result


def closure_retain_exact(path, data):
    # Re-entry inspects the original named effect. It never replaces a file or
    # generates a fresh name to hide an unknown/partial publication.
    if path.exists():
        require(read_regular(path, MAX_TAR, True) == data, f"retained closure effect differs: {path}")
    else:
        exclusive(path, data)


def parse_macho(data, *, binary=False):
    import struct
    require(len(data) >= 32 and data[:4] == b"\xcf\xfa\xed\xfe", "thin little-endian Mach-O64 required; fat/unknown image unsupported")
    _, cpu, subtype, filetype, count, commands_size, _, _ = struct.unpack_from("<8I", data)
    require(cpu == 16777228 and 0 < count <= 4096 and commands_size <= 256 * 1024 and len(data) >= 32 + commands_size, "Mach-O CPU/load-command bounds")
    require(filetype == (2 if binary else 6), "unexpected Mach-O file type")
    if binary:
        require(subtype == 0, "candidate ARM64 subtype must be exactly zero")
    uuid = None
    install = None
    loads, rpaths = [], []
    text_address = None
    position = 32
    def string_at(command, offset):
        require(8 <= offset < len(command), "Mach-O string offset")
        value, end, _ = command[offset:].partition(b"\x00")
        require(end and 0 < len(value) <= 4096, "Mach-O string terminator/bound")
        return link_map_path(value)
    for _ in range(count):
        require(position + 8 <= 32 + commands_size, "truncated Mach-O command")
        cmd, size = struct.unpack_from("<2I", data, position)
        require(size >= 8 and size % 8 == 0 and position + size <= 32 + commands_size, "invalid Mach-O command size")
        command_bytes = data[position:position + size]
        if cmd == 0x1b:  # LC_UUID; actual bytes, never a version-derived UUID.
            require(size == 24 and uuid is None, "missing/duplicate/invalid LC_UUID")
            uuid = command_bytes[8:24].hex()
        elif cmd == 0xd:  # LC_ID_DYLIB
            require(size >= 24 and install is None and not binary, "invalid LC_ID_DYLIB")
            install = string_at(command_bytes, struct.unpack_from("<I", command_bytes, 8)[0])
        elif cmd in (0xc, 0x80000018, 0x8000001f, 0x20, 0x80000023):
            require(size >= 24, "invalid dylib load command")
            loads.append(string_at(command_bytes, struct.unpack_from("<I", command_bytes, 8)[0]))
        elif cmd == 0x8000001c:  # LC_RPATH
            require(size >= 16, "invalid LC_RPATH")
            rpaths.append(string_at(command_bytes, struct.unpack_from("<I", command_bytes, 8)[0]))
        elif cmd == 0x19:  # LC_SEGMENT_64; __TEXT pins a cache image address.
            require(size >= 72, "invalid LC_SEGMENT_64")
            if command_bytes[8:24].split(b"\x00", 1)[0] == b"__TEXT":
                require(text_address is None, "duplicate __TEXT segment")
                text_address = struct.unpack_from("<Q", command_bytes, 24)[0]
        position += size
    require(position == 32 + commands_size and uuid is not None and uuid != "0" * 32, "Mach-O command/UUID completeness")
    require(len(loads) == len(set(loads)) <= 64 and len(rpaths) == len(set(rpaths)) <= 64, "duplicate/oversized Mach-O load or rpath list")
    return {"cpu_type": cpu, "cpu_subtype": subtype, "uuid": uuid, "fat": False, "rpaths": rpaths, "loads": loads}, install, text_address


def capture_dyld_context(run, prefix, sdk):
    """Read this builder's dyld metadata through SDK-declared Mach APIs only.

    TASK_DYLD_INFO/struct task_dyld_info: SDK mach/task_info.h:260-272;
    sharedCacheSlide/UUID/Base: SDK mach-o/dyld_images.h:128-176;
    mach_vm_read_overwrite: SDK mach/mach_vm.h:191. No target process,
    dlopen, injected code, helper controller, entitlement or guessed UUID.
    """
    import ctypes
    require(ctypes.sizeof(ctypes.c_void_p) == 8, "64-bit builder ABI required")
    headers = {}
    for relative in ("mach/task_info.h", "mach/mach_vm.h", "mach-o/dyld_images.h", "mach-o/loader.h", "mach-o/dyld.h"):
        headers[relative] = retain_blob(prefix, read_regular(Path(sdk) / "usr/include" / relative, platform_image=True))
    lib = ctypes.CDLL(None)  # Already loaded platform exports; no library load.
    try:
        task = ctypes.c_uint32.in_dll(lib, "mach_task_self_").value
        task_info = lib.task_info
        read_vm = lib.mach_vm_read_overwrite
    except (ValueError, AttributeError) as error:
        raise Refused("required SDK-declared own-task dyld metadata APIs unavailable") from error
    task_info.argtypes = [ctypes.c_uint32, ctypes.c_int32, ctypes.c_void_p, ctypes.POINTER(ctypes.c_uint32)]
    task_info.restype = ctypes.c_int32
    read_vm.argtypes = [ctypes.c_uint32, ctypes.c_uint64, ctypes.c_uint64, ctypes.c_uint64, ctypes.POINTER(ctypes.c_uint64)]
    read_vm.restype = ctypes.c_int32
    class TaskDyldInfo(ctypes.Structure):
        _pack_ = 4  # task_info.h:91 applies pack(push, 4).
        _fields_ = [("address", ctypes.c_uint64), ("size", ctypes.c_uint64), ("format", ctypes.c_int32)]
    class AllImagesPrefix(ctypes.Structure):
        _fields_ = [("version", ctypes.c_uint32), ("count", ctypes.c_uint32),
                    ("array", ctypes.c_void_p), ("notification", ctypes.c_void_p),
                    ("detached", ctypes.c_bool), ("initialized", ctypes.c_bool),
                    ("dyldImage", ctypes.c_void_p), ("jit", ctypes.c_void_p),
                    ("dyldVersion", ctypes.c_void_p), ("errorMessage", ctypes.c_void_p),
                    ("terminationFlags", ctypes.c_uint64), ("symbolication", ctypes.c_void_p),
                    ("systemOrder", ctypes.c_uint64), ("uuidCount", ctypes.c_uint64),
                    ("uuidArray", ctypes.c_void_p), ("allInfosAddress", ctypes.c_void_p),
                    ("initialCount", ctypes.c_uint64), ("errorKind", ctypes.c_uint64),
                    ("errorClient", ctypes.c_void_p), ("errorTarget", ctypes.c_void_p),
                    ("errorSymbol", ctypes.c_void_p), ("slide", ctypes.c_uint64),
                    ("uuid", ctypes.c_ubyte * 16), ("base", ctypes.c_uint64)]
    def read(address, length):
        require(type(address) is int and 0 < address < 2**64 and 0 < length <= 256 * 1024 and address + length < 2**64, "own-cache read bounds")
        buffer = ctypes.create_string_buffer(length)
        actual = ctypes.c_uint64()
        status = read_vm(task, address, length, ctypes.addressof(buffer), ctypes.byref(actual))
        require(status == 0 and actual.value == length, f"own-cache metadata read unsupported: kern_return={status} size={actual.value}")
        return buffer.raw
    def current():
        info = TaskDyldInfo()
        count = ctypes.c_uint32(ctypes.sizeof(info) // ctypes.sizeof(ctypes.c_uint32))
        status = task_info(task, 17, ctypes.byref(info), ctypes.byref(count))
        require(status == 0 and count.value == 5 and info.format == 1 and info.size >= ctypes.sizeof(AllImagesPrefix), "TASK_DYLD_INFO format/length unavailable")
        all_images = AllImagesPrefix.from_buffer_copy(read(info.address, ctypes.sizeof(AllImagesPrefix)))
        require(all_images.version >= 15 and all_images.base > 0 and all_images.initialized and not all_images.detached, "shared-cache identity unavailable/detached")
        uuid = bytes(all_images.uuid).hex()
        require(uuid != "0" * 32, "unknown shared-cache UUID")
        return {"shared_cache_uuid": uuid, "slide": all_images.slide, "base": all_images.base}
    snapshot = current()
    closure_retain_exact(run / "dyld-native-context.json", canonical({"sdk_headers": headers, "snapshot": snapshot, "api": "TASK_DYLD_INFO+mach_vm_read_overwrite on own task", "assumption": "SDK ABI and current platform dyld/shared-cache fidelity; no arbitrary-native theorem"}))
    # Some system dylibs (notably libsystem_kernel) also have universal disk
    # images with __TEXT at zero. dyld_info correctly reports every disk slice;
    # none may be mistaken for the actual mapped native platform image. Public
    # dyld.h APIs pin the already-loaded image's real header/slide/UUID instead.
    count_images = lib._dyld_image_count
    count_images.argtypes = []; count_images.restype = ctypes.c_uint32
    image_name = lib._dyld_get_image_name
    image_name.argtypes = [ctypes.c_uint32]; image_name.restype = ctypes.c_char_p
    image_header = lib._dyld_get_image_header
    image_header.argtypes = [ctypes.c_uint32]; image_header.restype = ctypes.c_void_p
    image_slide = lib._dyld_get_image_vmaddr_slide
    image_slide.argtypes = [ctypes.c_uint32]; image_slide.restype = ctypes.c_int64
    count = count_images()
    require(0 < count <= 4096, "own dyld image inventory bound")
    loaded = {}
    for index in range(count):
        raw_name = image_name(index)
        require(raw_name is not None and 0 < len(raw_name) <= 4096, "own dyld image name bound")
        name = raw_name.decode("utf8")
        if not (name.startswith("/usr/lib/") or name.startswith("/System/Library/")):
            continue
        address = image_header(index)
        header = read(address, 32)
        import struct
        length = struct.unpack_from("<I", header, 20)[0]
        require(length <= 256 * 1024 - 32, "own platform load-command bound")
        raw = read(address, 32 + length)
        macho, install, text = parse_macho(raw)
        slide = image_slide(index)
        require(install == name and text + slide == address and name not in loaded, "own platform mapping/identity mismatch")
        loaded[name] = {"address": address, "slide": slide, "macho": macho, "install": install, "text": text, "header": retain_blob(prefix, raw)}
    require(count_images() == count, "own image inventory changed during capture")
    closure_retain_exact(run / "own-platform-images.json", canonical(loaded))
    return snapshot, read, current, loaded


def emit_closure(run, prefix):
    """Return exact ICD Closure. Metadata only; never executes git or gpg.

    All recursive load edges are retained, including weak/reexport edges. A
    graph outside the ICD64-node/128-edge bounds refuses with original evidence.
    """
    import struct
    meta = strict_json(read_regular(run / "platform.json", 32768, True), 32768)
    sdk = Path(meta["sdk"])
    dyld_tool = Path("/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/bin/dyld_info")
    require(dyld_tool == dyld_tool.resolve(), "fixed dyld_info path required")
    dyld_identity = absolute_identity(dyld_tool)
    closure_retain_exact(run / "dyld-inspector.json", canonical(dyld_identity))
    snapshot, read_vm, current_cache, loaded_images = capture_dyld_context(run, prefix, sdk)
    _, os_output, _ = command(run, ["/usr/bin/sw_vers", "-buildVersion"])
    os_build = os_output.decode("ascii").strip()
    release, machine = platform.release(), platform.machine()
    require(re.fullmatch(r"[0-9]{2}[A-Z][0-9]{1,6}[a-z]?", os_build) is not None and re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,2}", release) is not None, "unknown OS build/release form")
    require(machine in ("arm64", "aarch64") and os_arch() == "aarch64", "unsupported platform architecture")
    platform_binding = {"arch": "aarch64", "darwin_machine": machine, "release": release, "os_build": os_build, "shared_cache_uuid": snapshot["shared_cache_uuid"]}
    records = strict_json(read_regular(run / "static-closure.json", 1024 * 1024, True), 1024 * 1024)
    require(type(records) is list and len(records) <= 64, "static closure bound")
    nodes, edges, link_maps = [], [], []
    image_ids, image_paths, pending = {}, {}, []
    archive_ids = {}
    evidence = []
    def add_edge(source, target, mode):
        edge = {"from": source, "to": target, "mode": mode}
        require(edge not in edges and len(edges) < 128, f"recursive closure duplicate/edge bound at {source} -> {target}; no truncation")
        edges.append(edge)
    for row in records:
        path = Path(row["original_path"])
        source = {"libgit.a": "git", "libgcrypt.a": "libgcrypt", "libassuan.a": "libassuan", "libnpth.a": "npth", "libgpg-error.a": "libgpg-error", "libksba.a": "libksba"}.get(path.name)
        if source is None:
            require(path.is_relative_to(run / "build/gnupg") and path.name in ("libkeybox.a", "libcommonpth.a", "libregexp.a", "libgpgrl.a"), "unreviewed linked static archive")
            source = "gnupg"
        data = pinned(read_regular(prefix / row["staged"], MAX_TAR, True), row["sha256"], "staged linked archive")
        node_id = "static-" + path.name.removesuffix(".a")
        require(node_id not in archive_ids.values(), "duplicate static-node identity")
        archive_ids[str(path)] = node_id
        nodes.append({"id": node_id, "kind": "static", "source": source, "file": {"path": row["staged"], "len": len(data), "sha256": sha(data)}, "macho": None, "member_index": row["member_index"], "install_name": None, "platform_uuid": None})
        ref_bytes(prefix, row["member_index"], 1024 * 1024)
    for name in ("git", "gpg"):
        raw = read_regular(run / "logs" / (name + ".link-map"), 8 * 1024 * 1024, True)
        parse_link_map_objects(raw, name)
        inputs = []
        for row in records:
            if name in row["selected_members"]:
                node_id = archive_ids[row["original_path"]]
                inputs.append(node_id)
                add_edge(name, node_id, "static")
        data = read_regular(prefix / "bin" / name, MAX_TAR, True)
        macho, install, _ = parse_macho(data, binary=True)
        require(install is None, "binary install name")
        nodes.append({"id": name, "kind": "binary", "source": name if name == "git" else "gnupg", "file": {"path": "bin/" + name, "len": len(data), "sha256": sha(data)}, "macho": macho, "member_index": None, "install_name": None, "platform_uuid": None})
        image_ids[str(prefix / "bin" / name)] = name
        image_paths[name] = prefix / "bin" / name
        pending.append((name, macho, prefix / "bin" / name))
        link_maps.append({"tool": name, "file": retain_blob(prefix, raw), "output_sha256": sha(data), "inputs": inputs})
    def system_path(path):
        return path.startswith("/usr/lib/") or path.startswith("/System/Library/")
    def resolve_load(load, owner, rpaths):
        if load.startswith("@loader_path/"):
            load = str(owner.parent / load[len("@loader_path/"):])
        elif load.startswith("@executable_path/"):
            load = str(prefix / "bin" / load[len("@executable_path/"):])
        elif load.startswith("@rpath/"):
            possible = []
            for rpath in rpaths:
                base = resolve_load(rpath, owner, [])
                value = posixpath.normpath(base + "/" + load[len("@rpath/"):])
                if system_path(value) or Path(value).is_file():
                    possible.append(value)
            require(len(possible) == 1, f"unresolved/ambiguous @rpath load: {load}")
            return possible[0]
        require(load.startswith("/") and "\x00" not in load and len(load.encode()) <= 4096, "unresolved/oversized image load")
        result = posixpath.normpath(load)
        require(system_path(result) or Path(result).is_relative_to(prefix), f"ambient/unapproved image load: {result}")
        return result
    while pending:
        owner_id, macho, owner_path = pending.pop(0)
        for load in macho["loads"]:
            path = resolve_load(load, owner_path, macho["rpaths"])
            target_id = image_ids.get(path)
            if target_id is None:
                require(len(nodes) < 64, f"recursive closure exceeds64 nodes at {path}; no omitted platform image")
                if system_path(path):
                    target_id = "platform-" + sha(path.encode())[:20]
                    _, raw, err = command(run, [str(dyld_tool), "-uuid", "-load_commands", "-inits", path])
                    require(not err and len(raw) <= 2 * 1024 * 1024, "dyld_info image metadata failed/oversized")
                    mapped = loaded_images.get(path)
                    # Select by the actual mapped UUID, never by an architecture
                    # nickname, first universal slice or an assumed cache path.
                    view = raw
                    if mapped is not None:
                        sections = re.split(rb"(?m)^" + re.escape(path.encode()) + rb" \[[^\]\n]+\]:\n", raw)[1:]
                        matching = [part for part in sections if mapped["macho"]["uuid"].encode() in part.replace(b"-", b"").lower()]
                        require(len(matching) == 1, "actual mapped platform UUID absent/ambiguous in inspector output")
                        view = matching[0]
                    match = re.search(rb'segname: "__TEXT"\s+vmaddr: (0x[0-9a-fA-F]+)', view)
                    uuids = re.findall(rb"^\s*uuid: ([0-9A-Fa-f-]{36})\s*$", view, re.MULTILINE)
                    require(match is not None and len(uuids) == 1, "dyld_info missing/ambiguous actual __TEXT/UUID")
                    # Cache construction may rewrite __TEXT addresses while
                    # retaining the slice UUID. The public loaded-image API,
                    # not the disk layout at vmaddr zero, owns that address.
                    text_address_expected = mapped["text"] if mapped is not None else int(match[1], 16)
                    address = mapped["address"] if mapped is not None else text_address_expected + snapshot["slide"]
                    require(mapped is not None or address >= snapshot["base"], "cache image lies before actual cache mapping")
                    header = read_vm(address, 32)
                    require(header[:4] == b"\xcf\xfa\xed\xfe", "cache image header unavailable")
                    length = struct.unpack_from("<I", header, 20)[0]
                    require(length <= 256 * 1024 - 32, "cache Mach-O command bound")
                    cache_bytes = read_vm(address, 32 + length)
                    actual, install, text_address = parse_macho(cache_bytes)
                    require(install == path and text_address == text_address_expected and actual["uuid"] == uuids[0].replace(b"-", b"").decode().lower(), "platform image/install-name/UUID mismatch")
                    node = {"id": target_id, "kind": "platform", "source": None, "file": None, "macho": actual, "member_index": None, "install_name": install, "platform_uuid": actual["uuid"]}
                    evidence.append({"image": path, "dyld_info": retain_blob(prefix, raw), "macho_header_commands": retain_blob(prefix, cache_bytes)})
                else:
                    relative = str(Path(path).relative_to(prefix))
                    data = read_regular(path, MAX_TAR, True)
                    actual, install, _ = parse_macho(data)
                    require(install is not None, "private dylib install name missing")
                    # This recipe currently builds all private libraries static;
                    # an unexpected dylib needs an exact source/build mapping.
                    raise Refused(f"unexpected private dylib requires source/link review: {relative} ({install})")
                require(target_id not in image_paths, "image-ID hash collision")
                nodes.append(node)
                image_ids[path] = target_id
                image_paths[target_id] = Path(path)
                pending.append((target_id, actual, Path(path)))
            add_edge(owner_id, target_id, "load")
    # A linker can obtain symbols through a platform reexport. Preserve those
    # actual SDK stub inputs as well as static archives; do not turn their
    # presence into invented direct LC_LOAD edges from the executable.
    for link_map in link_maps:
        raw = ref_bytes(prefix, link_map["file"], 8 * 1024 * 1024)
        for entry in parse_link_map_objects(raw, link_map["tool"]):
            if not entry.endswith(b".tbd"):
                continue
            stub = Path(link_map_path(entry)).resolve()
            require(stub.is_relative_to(sdk.resolve()), "link map names an ambient SDK stub")
            data = read_regular(stub, 16 * 1024 * 1024, platform_image=True)
            names = re.findall(rb"^install-name:\s*'([^'\x00\r\n]+)'\s*$", data, re.MULTILINE)
            require(len(names) == 1, "unsupported/ambiguous SDK stub install name")
            install_name = link_map_path(names[0])
            target_id = image_ids.get(install_name)
            require(target_id is not None, f"link-map SDK input absent from actual recursive loaded closure: {install_name}")
            if target_id not in link_map["inputs"]:
                require(len(link_map["inputs"]) < 64, "link-map input-node bound")
                link_map["inputs"].append(target_id)
            evidence.append({"sdk_stub": str(stub), "file": retain_blob(prefix, data), "install_name": install_name})
    require(current_cache() == snapshot, "dyld shared-cache identity changed during closure capture")
    require(absolute_identity(dyld_tool) == dyld_identity, "metadata inspector changed during closure capture")
    require(len(nodes) <= 64 and len(edges) <= 128, "recursive closure exceeds typed budget")
    closure_retain_exact(run / "platform-image-evidence.json", canonical(evidence))
    closure = {"nodes": nodes, "edges": edges, "link_maps": link_maps, "platform": platform_binding}
    closure_retain_exact(run / "closure.json", canonical(closure))
    return closure


def harden_owned_tree(root):
    root = Path(root)
    for current, dirs, files in os.walk(root, followlinks=False):
        path = Path(current)
        require(path.lstat().st_uid == os.getuid() and not path.is_symlink(), "foreign scratch directory")
        path.chmod(0o700)
        for name in dirs + files:
            child = path / name
            info = child.lstat()
            require(info.st_uid == os.getuid(), "foreign scratch artifact")
            if stat.S_ISLNK(info.st_mode):
                require(child.resolve().is_relative_to(root), "escaping private dependency symlink")
            elif stat.S_ISDIR(info.st_mode):
                child.chmod(0o700)
            else:
                require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1, "scratch special/hardlink artifact")
                child.chmod(0o700 if info.st_mode & 0o111 else 0o600)


def supported_option(flag, helpout):
    option = flag.split("=")[0]
    return option.encode() in helpout or option.replace("--disable-", "--enable-").replace("--without-", "--with-").encode() in helpout


def apply_profile(run):
    patches = json.loads(read_regular(FIX / "patches.lock.json", 32768))
    for patch in patches:
        work = run / "src" / patch["source"]
        data = pinned(read_regular(FIX / patch["patch"], 32768), patch["sha256"], "profile patch")
        for entry in patch["files"]:
            pinned(read_regular(work / entry["path"]), entry["before"], "patch source")
        # Exact source preimages and postimages rule out offset/fuzzy adoption.
        local = run / "logs" / patch["patch"]
        exclusive(local, data)
        command(run, ["/usr/bin/patch", "-p1", "-F0", "--batch", "--forward", "-i", str(local)], cwd=work)
        for entry in patch["files"]:
            pinned(read_regular(work / entry["path"]), entry["after"], "patched source")


def blob_ref(data):
    return {"path": "provenance/blobs/" + sha(data), "len": len(data), "sha256": sha(data)}


def retain_blob(prefix, data):
    private_dir(prefix / "provenance")
    private_dir(prefix / "provenance/blobs")
    ref = blob_ref(data)
    path = prefix / ref["path"]
    if path.exists():
        require(read_regular(path, MAX_TAR, True) == data, "existing provenance blob differs")
    else:
        exclusive(path, data)
    return ref


def retained_file(prefix, path):
    return retain_blob(prefix, read_regular(path, MAX_TAR, True))


def absolute_identity(path):
    path = Path(path)
    require(path.is_absolute() and path == path.resolve(), "fixed resolved absolute image required")
    data = read_regular(path, MAX_TAR, platform_image=True)
    s = path.stat()
    return {"path": str(path), "len": len(data), "sha256": sha(data), "dev": s.st_dev, "ino": s.st_ino,
            "mtime_sec": s.st_mtime_ns // 1000000000, "mtime_nsec": s.st_mtime_ns % 1000000000}


def dir_pin(path):
    s = check_private_dir(path).stat()
    return {"path": str(path), "dev": s.st_dev, "ino": s.st_ino, "uid": s.st_uid}


def file_pin(root, path, kind="candidate"):
    data = read_regular(root / path, MAX_TAR, True)
    s = (root / path).stat()
    return {"root": kind, "path": str(path), "len": len(data), "sha256": sha(data), "dev": s.st_dev, "ino": s.st_ino,
            "mtime_sec": s.st_mtime_ns // 1000000000, "mtime_nsec": s.st_mtime_ns % 1000000000}


def command_profile():
    # Exact profile65126. This is producer data; Q2 owns its sole strict
    # canonical consumer/decoder and actual command execution semantics.
    forms = [
        ("root_bare", ["rev-parse", "--is-bare-repository"]),
        ("root_top", ["rev-parse", "--show-toplevel"]),
        ("policy_main", ["rev-parse", "--verify", "{policy_branch}"]),
        ("origin_main", ["rev-parse", "--verify", "refs/remotes/origin/main"]),
        ("object_type", ["cat-file", "-t", "{commit}"]),
        ("ancestry", ["merge-base", "--is-ancestor", "{ancestor}", "{commit}"]),
        ("tree_record", ["ls-tree", "-z", "{commit}", "--", "{record}"]),
        ("show_record", ["show", "{commit}:{record}"]),
        ("last_touch", ["rev-list", "-1", "{commit}", "--", "{record}"]),
        ("blob_pin", ["rev-parse", "{commit}:{record}"]),
        ("tree_pin", ["rev-parse", "{commit}^{tree}"]),
        ("origin_url", ["config", "--get", "remote.origin.url"]),
        ("signature", ["log", "-1", "--format=%G?%x00%GF%x00%GP", "{commit}"]),
    ]
    result = [{"id": name, "tool": "git", "argv": argv, "members": 2 if name == "signature" else 1, "stdin": "none"} for name, argv in forms]
    result += [{"id": "show_keys", "tool": "gpg", "argv": ["--with-colons", "--show-keys", "-"], "members": 1, "stdin": "public_keys"},
               {"id": "import_keys", "tool": "gpg", "argv": ["--import", "-"], "members": 1, "stdin": "public_keys"},
               # Matcher only: Q2 binds Git's own temporary at GS1/GS2. The
               # builder never expands it or launches this implicit form.
               {"id": "verify_child", "tool": "gpg", "argv": ["--keyid-format=long", "--status-fd=1", "--verify", GIT_SIGNATURE_TEMP, "-"], "members": 1, "stdin": "commit_payload"}]
    return {"profile": "65126", "git_prefix": ["--no-pager", "--no-optional-locks", "-c", "core.fsmonitor=false", "-c", "core.hooksPath=/dev/null", "-c", "credential.helper=", "-c", "protocol.allow=never", "-c", "gpg.format=openpgp", "-c", "gpg.program={gpg}", "-c", "gpg.openpgp.program={gpg}"],
            "gpg_prefix": ["--no-options", "--no-autostart", "--batch", "--no-tty"], "forms": result}


def private_config(run, prefix):
    home = private_dir(run / "fixture-home")
    private_dir(run / "fixture-scratch")
    config = b"batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n"
    exclusive(home / "gpg.conf", config)
    exclusive(home / "common.conf", b"")
    values = [("GIT_CONFIG_NOSYSTEM", "1"), ("GIT_CONFIG_GLOBAL", "/dev/null"), ("GIT_TERMINAL_PROMPT", "0"),
              ("GIT_ASKPASS", "/usr/bin/false"), ("SSH_ASKPASS", "/usr/bin/false"), ("LANG", "C"), ("LC_ALL", "C"),
              ("PATH", "/usr/bin:/bin"), ("HOME", "{home}"), ("GNUPGHOME", "{home}"), ("TMPDIR", "{scratch}")]
    return {"gpg_conf": retain_blob(prefix, config), "common_conf": retain_blob(prefix, b""),
            "env": [{"name": k, "value": v} for k, v in values]}


def git_sha1_commit_parts(commit):
    """Unfold these fixed SHA-1 fixtures without altering a single LF/byte.

    Inspired by Git2.54.0 commit.c:1186-1229 (GPL-2.0): header records select
    gpgsig; every gpgsig namespace record is omitted from the signed payload.
    This independent byte-format check does not parse/verify OpenPGP or replace
    Git. Original source: https://github.com/git/git/blob/v2.54.0/commit.c
    Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
    """
    require(type(commit) is bytes and len(commit) <= 2*1024*1024, "fixture commit byte bound")
    records = []
    stream = io.BytesIO(commit)
    remainder = b""
    for line in stream:
        if line == b"\n":
            remainder = line + stream.read()
            break
        if line.startswith(b" ") and records:
            records[-1].append(line)
        else:
            records.append([line])
    signature, payload = [], []
    for record in records:
        if record[0].startswith(b"gpgsig "):
            signature.append(record[0][7:])
            signature.extend(line[1:] for line in record[1:])
        elif not record[0].startswith(b"gpgsig"):
            payload.extend(record)
    return b"".join(payload) + remainder, b"".join(signature)


def corpus_signature_bindings(data, lock):
    refs = {row["path"]: row for row in lock["files"]}
    require(len(refs) == len(lock["files"]), "duplicate corpus Ref")
    needed = {"valid.commit", "corrupt.commit", "valid-signature.asc", "corrupt-signature.asc", "valid-payload.txt"}
    require(needed <= set(refs) and needed <= set(data), "missing corpus case Ref/bytes")
    for name in needed:
        validate_ref(refs[name])
    results = []
    for case in ("valid", "corrupt"):
        commit = data[case + ".commit"]
        body = b"commit " + str(len(commit)).encode() + b"\0" + commit
        require(hashlib.sha1(body).hexdigest() == lock[case + "_commit"], "case commit object binding")
        payload, extracted = git_sha1_commit_parts(commit)
        expected = refs[case + "-signature.asc"]
        require(extracted == data[expected["path"]] and len(extracted) == expected["len"] and sha(extracted) == expected["sha256"], "Git-extracted signature/case Ref mismatch")
        payload_ref = refs["valid-payload.txt"]
        require(payload == data[payload_ref["path"]] and len(payload) == payload_ref["len"] and sha(payload) == payload_ref["sha256"], "Git-extracted payload Ref mismatch")
        results.append({"case": case, "commit": lock[case + "_commit"], "signature": expected, "extracted_len": len(extracted), "extracted_sha256": sha(extracted), "payload": payload_ref})
    require(results[0]["signature"]["sha256"] != results[1]["signature"]["sha256"], "corrupt proof must retain distinct actual signature bytes")
    return results


def read_corpus():
    corpus = FIX / "corpus"
    lock = strict_json(read_regular(corpus / "corpus.lock.json", 32768), 32768)
    data = {}
    for row in lock["files"]:
        data[row["path"]] = pinned(read_regular(corpus / row["path"], 2*1024*1024), row["sha256"], "public fixture")
        require(len(data[row["path"]]) == row["len"], "public fixture length")
    require(b"PRIVATE KEY" not in data["public-keys.asc"] and b"SECRET KEY" not in data["public-keys.asc"], "public-only fixture required")
    corpus_signature_bindings(data, lock)
    return data, lock


def prepare_fixture(run):
    # Copied/adapted from Git v2.54.0 t7510's deterministic presigned objects.
    # Origin/terms: https://github.com/git/git/blob/v2.54.0/t/t7510-signed-commit.sh
    # Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
    # Construct loose objects with their specified bytes; never invoke signing,
    # key generation, a secret packet, or an ambient Git configuration.
    data, lock = read_corpus()
    repo = private_dir(run / "fixture-repo")
    private_dir(repo / ".git")
    private_dir(repo / ".git/objects")
    private_dir(repo / ".git/refs"); private_dir(repo / ".git/refs/heads")
    private_dir(repo / ".git/refs/remotes"); private_dir(repo / ".git/refs/remotes/origin")
    def obj(kind, name):
        body = data[name]
        raw = kind.encode() + b" " + str(len(body)).encode() + b"\0" + body
        oid = hashlib.sha1(raw).hexdigest()
        private_dir(repo / ".git/objects" / oid[:2])
        exclusive(repo / ".git/objects" / oid[:2] / oid[2:], zlib.compress(raw))
        return oid
    obj("blob", "first.blob")
    require(obj("blob", "second.blob") == lock["record_blob"], "record blob differs")
    obj("tree", "first.tree")
    require(obj("tree", "selected.tree") == lock["selected_tree"], "tree differs")
    for name in ("ancestor", "valid", "corrupt"):
        require(obj("commit", name + ".commit") == lock["ancestor" if name == "ancestor" else name + "_commit"], "commit fixture differs")
    branch = "refs/heads/proof-main"
    origin = "https://example.invalid/bboxy450/private-presigned-fixture.git"
    exclusive(repo / ".git/HEAD", ("ref: " + branch + "\n").encode())
    exclusive(repo / (".git/" + branch), (lock["valid_commit"] + "\n").encode())
    exclusive(repo / ".git/refs/remotes/origin/main", (lock["valid_commit"] + "\n").encode())
    exclusive(repo / ".git/config", ("[core]\n\trepositoryformatversion = 0\n\tbare = false\n[remote \"origin\"]\n\turl = " + origin + "\n").encode())
    exclusive(repo / "first.t", data["first.blob"])
    exclusive(repo / "second", data["second.blob"])
    for name in ("public-keys.asc", "valid-payload.txt", "valid-signature.asc", "corrupt-signature.asc"):
        exclusive(repo / name, data[name])
    files = []
    for path in sorted(repo.rglob("*"), key=lambda p: p.relative_to(repo).as_posix().encode()):
        if path.is_dir():
            check_private_dir(path)
        else:
            raw = read_regular(path, 2*1024*1024, True)
            files.append({"path": path.relative_to(repo).as_posix(), "len": len(raw), "sha256": sha(raw)})
    require(len(files) <= 32, "private fixture inventory exceeds32 files")
    bypath = {row["path"]: row for row in files}
    return {"repo": dir_pin(repo), "files": files, "ancestor": lock["ancestor"], "valid_commit": lock["valid_commit"], "corrupt_commit": lock["corrupt_commit"],
            "policy_branch": branch, "record_path": lock["record_path"], "record_blob": lock["record_blob"], "origin_url": origin,
            "public_keys": bypath["public-keys.asc"], "signer_primary": lock["signer_primary"], "valid_payload": bypath["valid-payload.txt"],
            "valid_signature": bypath["valid-signature.asc"], "corrupt_signature": bypath["corrupt-signature.asc"]}


def review_findings(run, patches):
    template = strict_json(read_regular(FIX / "review.lock.json", 32768), 32768)
    snapshot = strict_json(read_regular(run / "source-snapshot.json", MAX_TAR, True), MAX_TAR)
    names = {name: name + "-" + version for name, version, *_ in PINS}
    findings = []
    for finding in template:
        row = dict(finding)
        spans = []
        for span in row["spans"]:
            path = names[span["source"]] + "/" + span["path"]
            raw = read_regular(run / "src" / path, MAX_ARCHIVE, True)
            require(snapshot[path]["sha256"] == sha(raw), "review span no longer matches authenticated postimage")
            require(1 <= span["first"] <= span["last"] <= len(io.BytesIO(raw).readlines()), "source review line span")
            spans.append({**span, "sha256": sha(raw)})
        row["spans"] = spans
        row["patches"] = [p["sha256"] for p in patches if p["source"] in [names[s] for s in row["sources"]]]
        findings.append(row)
    coverage = {"git_async": ("deny_async", {"git"}), "git_child": ("one_verify_child", {"git", "gnupg"}),
                "git_shell": ("deny_shell", {"git"}), "gpg_service": ("deny_service", {"gnupg", "libgpg-error", "libassuan"}),
                "gpg_thread": ("deny_thread", {"gnupg", "libgcrypt", "npth"}),
                "libraries": ("review_only", {p[0] for p in PINS[2:]}), "input_config": ("fixed_inputs", {"git", "gnupg"}),
                "platform": ("review_only", {"git", "gnupg"})}
    require(len(findings) == 8 and {f["id"] for f in findings} == set(coverage), "source review eight findings")
    for row in findings:
        guard, required = coverage[row["id"]]
        require(row["guard"] == guard and set(row["sources"]) >= required and set(row["sources"]) <= {s["source"] for s in row["spans"]}, "source review guard/span coverage")
        require(row["phases"] == ["entry", "normal", "error", "cleanup", "initializer"] and row["finding"] == "conditional_source_review", "source review phases/conditional premise")
        require(0 < len(row["spans"]) <= 16 and len(row["rationale"].encode()) <= 512, "source review span/rationale bounds")
    require(set().union(*(set(f["commands"]) for f in findings)) == {f["id"] for f in command_profile()["forms"]}, "source review command coverage")
    return findings


def capture_review(run, projections):
    require(projections["commands"] == command_profile(), "stale/substituted Commands projection; Git temporary matcher required")
    return {"bindings": {key: sha(canonical(value)) for key, value in projections.items()}, "findings": review_findings(run, projections["build"]["patches"])}


def check_signature_corpus(d):
    # B-only source/corpus verification. No candidate, build, authentication,
    # real Git/GPG, probe, timing gate or GS1/GS2 implementation is invoked.
    unit(d, signature_only=True)
    run = session(d, "signature-corpus")
    rows = []
    snapshot = strict_json(read_regular(SIGNATURE_SOURCE_RUN / "source-snapshot.json", MAX_TAR, True), MAX_TAR)
    for name, digest in SIGNATURE_SOURCE_PINS.items():
        path = SIGNATURE_SOURCE_RUN / "src/git-2.54.0" / name
        require(snapshot["git-2.54.0/" + name]["sha256"] == digest, "B source snapshot binding")
        source = pinned(read_regular(path, MAX_ARCHIVE, True), digest, "B authenticated Git source")
        rows.append({"path": str(path), "len": len(source), "sha256": digest})
    data, lock = read_corpus()
    commands = command_profile()
    patches = strict_json(read_regular(FIX / "patches.lock.json", 32768), 32768)
    findings = review_findings(SIGNATURE_SOURCE_RUN, patches)
    record = {"source_files": rows, "corpus_lock_sha256": sha(read_regular(FIX / "corpus/corpus.lock.json", 32768)),
              "cases": corpus_signature_bindings(data, lock), "commands_sha256": sha(canonical(commands)),
              "review_template_sha256": sha(read_regular(FIX / "review.lock.json", 32768)), "review_findings_sha256": sha(canonical(findings)),
              "review_commands_binding_sha256": sha(canonical(commands)), "full_review_projection_sha256": None,
              "source_extraction_equivalence": "fixed_SHA1_corpus_bytes", "cryptographic_verification": "not_run", "GS1_GS2": "Q2_owned_not_run"}
    exclusive(run / "signature-corpus.json", canonical(record))
    print(f"B source/corpus byte equivalence retained at {run}; commands={record['commands_sha256']}; real verifier witness remains unproven", flush=True)


def prepare_basis(run, prefix, closure):
    intent = strict_json(read_regular(run / "intent.json", 32768, True), 32768)
    inputs = intent["inputs"]
    require(read_regular(ROOT / "scripts/dream-proof-tools.py") == SCRIPT_BYTES and sha(SCRIPT_BYTES) == inputs["source_sha256"], "builder recipe changed during its original build")
    actual_fixtures = {str(p.relative_to(FIX)): sha(read_regular(p, MAX_TAR)) for p in sorted(FIX.rglob("*")) if p.is_file()}
    require(actual_fixtures == inputs["fixtures"], "source/key/patch/review/fixture tuple changed during original build")
    metadata = strict_json(read_regular(run / "platform.json", 32768, True), 32768)
    # Capture exact actual outputs, not versions inferred from archive labels.
    def output(argv):
        _, out, err = command(run, argv)
        return retain_blob(prefix, out + (b"\nSTDERR\n" + err if err else b""))
    compiler = Path(metadata["compiler"]).resolve()
    linker = Path(metadata["linker"]).resolve()
    compiler_version = output([str(compiler), "--version"])
    _, target, _ = command(run, [str(compiler), "-dumpmachine"])
    reported = target.decode("ascii").strip()
    require(re.fullmatch(r"(?:arm64|aarch64)-apple-darwin[0-9]+(?:\.[0-9]+){0,2}", reported) and reported.split("darwin")[1] == platform.release(), "compiler target/platform differs")
    linker_version = output([str(linker), "-version_details"])
    auth = strict_json(read_regular(run / "authenticated.json", 32768, True), 32768)
    auth = [{**row, "status": retained_file(prefix, run / "public-home" / (row["name"] + ".status"))} for row in auth]
    patch_data = read_regular(FIX / "patches.lock.json", 32768)
    patches = strict_json(patch_data, 32768)
    # Actual --help/configure/make invocations and their cleared environments
    # are archived. Existing immutable records precede this metadata capture.
    requests = [strict_json(read_regular(path, MAX_ARCHIVE, True), MAX_ARCHIVE) for path in sorted((run / "logs").glob("*.command.json"))]
    recipe_commands = retain_blob(prefix, canonical(requests))
    recipe_env = retain_blob(prefix, canonical([row["env"] for row in requests]))
    generated = []
    for name, version, *_ in PINS:
        if name == "git":
            config = run / "src/git-2.54.0/GIT-CFLAGS"
            summary = canonical({filename: retained_file(prefix, run / "src/git-2.54.0" / filename) for filename in ("config.mak.uname", "GIT-LDFLAGS", "GIT-PREFIX", "GIT-VERSION-FILE")})
        else:
            config = run / "build" / name / "config.h"
            summary = read_regular(run / "build" / name / "config.log", MAX_TAR, True)
        generated.append({"source": name, "config": retained_file(prefix, config), "summary": retain_blob(prefix, summary)})
    lock = retain_blob(prefix, read_regular(FIX / "sources.lock.json", 32768))
    for row in source_lock()["sources"]:
        keyname = "git" if row["name"] == "git" else "gnupg"
        retain_blob(prefix, read_regular(FIX / (keyname + "-release-key.asc")))
    for patch in patches:
        retain_blob(prefix, read_regular(FIX / patch["patch"], 32768))
    retain_blob(prefix, pinned(read_regular(AUTHORITY / "proof-helper-probe.c", 39130), PROBE_SHA, "current scoped frozen source"))
    build = {"build_id": run.name, "incarnation": intent["incarnation"], "builder": {"commit": inputs["commit"], "source_sha256": inputs["source_sha256"]},
             "source_lock": lock, "auth": auth, "patch_lock": retain_blob(prefix, patch_data), "patches": patches,
             "recipe": {"file": retained_file(prefix, run / "recipe.py"), "commands": recipe_commands, "env": recipe_env},
             "compiler": {"image": absolute_identity(compiler), "version": compiler_version, "target": "aarch64-apple-darwin", "reported_target": reported},
             "linker": {"image": absolute_identity(linker), "version": linker_version},
             "sdk": {"path": inputs["sdk"], "canonical_name": "macosx" + inputs["sdk_version"], "version": inputs["sdk_version"], "settings": retain_blob(prefix, read_regular(Path(inputs["sdk"]) / "SDKSettings.json"))},
             "platform": closure["platform"], "generated": generated, "tools": {name: absolute_identity(prefix / "bin" / name) for name in ("git", "gpg")}}
    projections = {"build": build, "closure": closure, "config": private_config(run, prefix), "commands": command_profile(), "fixture": prepare_fixture(run)}
    basis = {"format": "bboxy.dream.private-pretest-basis/1", **projections, "review": capture_review(run, projections)}
    raw = canonical(basis)
    # Reserve the full canonical result ceiling and descriptor ceiling now.
    # This is a reservation, not permission to fill past the aggregate cap.
    require(len(raw) <= 49152 and len(raw) + lock["len"] + 65536 + 16384 <= 131072, "pre-test basis/result/descriptor reservation exceeds128KiB")
    exclusive(prefix / "provenance/pretest-basis.json", raw)
    exclusive(run / "basis-ready.json", canonical({"basis": file_pin(prefix, "provenance/pretest-basis.json"), "source_lock": file_pin(prefix, lock["path"]), "probe_admitted": False}))
    return basis


def source_lock():
    sources = []
    for name, version, url, digest, sigsha in PINS:
        keyname = "git" if name == "git" else "gnupg"
        key = pinned(read_regular(FIX / (keyname + "-release-key.asc"), MAX_ARCHIVE), KEYS[keyname][1], "public key")
        sources.append({"name": name, "version": version, "archive_url": url, "archive_sha256": digest,
                        "signature_url": url[:-3] + ".sign" if name == "git" else url + ".sig", "signature_sha256": sigsha,
                        "compression": "xz" if name == "git" else "bz2", "signed_bytes": "uncompressed_tar" if name == "git" else "archive",
                        "roots": [GIT_ROOT] if name == "git" else GPG_ROOTS, "signing_subkeys": [GIT_SUB] if name == "git" else GPG_ROOTS,
                        "public_key": blob_ref(key)})
    patches = json.loads(read_regular(FIX / "patches.lock.json", 32768))
    return {"format": "bboxy.dream.proof-tool-sources/1", "sources": sources, "backports": [],
            "profile_patches": [blob_ref(pinned(read_regular(FIX / p["patch"], 32768), p["sha256"], "profile patch")) for p in patches],
            "bootstrap_verifier": {"absolute_path": VERIFIER, "sha256": VERIFIER_SHA},
            "canonical_q2": {"source": {"path": "provenance/blobs/" + PROBE_SHA, "len": 39130, "sha256": PROBE_SHA}, "sha256": PROBE_SHA}}


def strict_json(data, limit):
    require(len(data) <= limit, "JSON byte budget")
    def unique(pairs):
        out = {}
        for key, value in pairs:
            require(key not in out, "duplicate JSON key")
            out[key] = value
        return out
    try:
        value = json.loads(data.decode("utf-8"), object_pairs_hook=unique, parse_constant=lambda _: (_ for _ in ()).throw(Refused("nonfinite JSON")))
    except (ValueError, UnicodeError) as error:
        raise Refused(f"invalid JSON: {error}") from error
    require(canonical(value) == data, "noncanonical JSON")
    return value


def fields(value, names):
    require(isinstance(value, dict) and set(value) == set(names.split()), "unknown/missing fields")


def bounded_string(value, limit):
    require(type(value) is str and 0 < len(value.encode()) <= limit and "\x00" not in value, "string bound/type")


def hex_string(value, length=64, uppercase=False):
    alphabet = "0-9A-F" if uppercase else "0-9a-f"
    require(type(value) is str and re.fullmatch(f"[{alphabet}]{{{length}}}", value) is not None, "digest/fingerprint form")


def checked_int(value, minimum=0, maximum=MAX_INT):
    require(type(value) is int and minimum <= value <= maximum, "checked integer bound/type")


def checked_array(value, limit=64):
    require(type(value) is list and len(value) <= limit, "array bound/type")


def validate_ref(value):
    fields(value, "path len sha256")
    bounded_string(value["path"], 4096)
    require(not value["path"].startswith("/") and all(p not in ("", ".", "..") for p in value["path"].split("/")), "relative fixed reference required")
    checked_int(value["len"])
    hex_string(value["sha256"])


def ref_bytes(prefix, ref, limit=MAX_TAR):
    validate_ref(ref)
    fd = os.open(prefix, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        for part in ref["path"].split("/")[:-1]:
            nextfd = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
            os.close(fd); fd = nextfd
            s = os.fstat(fd)
            require(s.st_uid == os.getuid() and stat.S_IMODE(s.st_mode) == 0o700, "reference directory custody")
        leaf = os.open(ref["path"].split("/")[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
        try:
            s = os.fstat(leaf)
            require(stat.S_ISREG(s.st_mode) and s.st_nlink == 1 and s.st_uid == os.getuid() and stat.S_IMODE(s.st_mode) in (0o600, 0o700) and s.st_size == ref["len"] <= limit, "reference file custody/length")
            out = bytearray()
            while len(out) <= limit:
                chunk = os.read(leaf, min(65536, limit + 1 - len(out)))
                if not chunk: break
                out.extend(chunk)
            t = os.fstat(leaf)
            require((s.st_dev, s.st_ino, s.st_size, s.st_mtime_ns, s.st_ctime_ns) == (t.st_dev, t.st_ino, t.st_size, t.st_mtime_ns, t.st_ctime_ns), "reference changed during hash")
            return pinned(bytes(out), ref["sha256"], ref["path"])
        finally:
            os.close(leaf)
    finally:
        os.close(fd)


def validate_lock(data):
    lock = strict_json(data, 32768)
    # Equality to independently frozen pins includes every known field, order,
    # release root, signed byte form, key, patch preimage and canonical probe.
    require(lock == source_lock(), "source/key/patch/probe lock mismatch")
    return lock


def validate_proposal(data):
    receipt = strict_json(data, 32768)
    fields(receipt, "schema source_sha256 os git gpg capability")
    require(receipt["schema"] == "bboxy.dream.proof-tools/1" and receipt["source_sha256"] == Q2_SHA, "proposal source/schema mismatch")
    fields(receipt["os"], "release arch")
    for value in receipt["os"].values(): bounded_string(value, 128)
    for name in ("git", "gpg"):
        tool = receipt[name]; fields(tool, "absolute_path sha256 build features")
        bounded_string(tool["absolute_path"], 4096)
        require(tool["absolute_path"].startswith("/"), "absolute tool required")
        hex_string(tool["sha256"]); bounded_string(tool["build"], 4096)
        checked_array(tool["features"])
        for value in tool["features"]: bounded_string(value, 128)
    capability = receipt["capability"]
    fields(capability, "probe_source_sha256 probe_exe_sha256 evidence_sha256 features")
    for key in ("probe_source_sha256", "probe_exe_sha256", "evidence_sha256"): hex_string(capability[key])
    require(capability["probe_source_sha256"] == PROBE_SHA, "canonical probe mismatch")
    checked_array(capability["features"])
    for value in capability["features"]: bounded_string(value, 128)
    return receipt


def validate_qualification(data, prefix):
    q = strict_json(data, 131072)
    fields(q, "format build_id incarnation builder source_lock sources patches compiler sdk platform tools loaded_closure configs command_profile_sha256 review component checks logs candidate_status")
    require(q["format"] == "bboxy.dream.proof-tool-qualification/1", "qualification format")
    require(type(q["build_id"]) is str and re.fullmatch("[0-7][0-9A-HJKMNP-TV-Z]{25}", q["build_id"]), "canonical build ULID")
    checked_int(q["incarnation"], 1)
    fields(q["builder"], "commit source_sha256"); hex_string(q["builder"]["commit"], 40); hex_string(q["builder"]["source_sha256"])
    refs = [q["source_lock"]]
    lock_data = ref_bytes(prefix, q["source_lock"], 32768); validate_lock(lock_data)
    fields(q["component"], "authority_source probe_exe report")
    refs += list(q["component"].values())
    report = ref_bytes(prefix, q["component"]["report"], 131072)
    prepared_budget(data, lock_data, report)
    fields(q["tools"], "git gpg"); refs += list(q["tools"].values())
    checked_array(q["sources"]); require(len(q["sources"]) == len(PINS), "exact source authentication count required")
    for row, source in zip(q["sources"], PINS):
        fields(row, "name archive_sha256 signature_sha256 signer_primary signer_subkey verifier_sha256 auth_exit auth_status_sha256")
        checked_int(row["auth_exit"], -(2**31), 2**31-1)
        require(row["name"] == source[0] and row["archive_sha256"] == source[3] and row["signature_sha256"] == source[4] and row["verifier_sha256"] == VERIFIER_SHA and row["auth_exit"] == 0, "source authentication binding")
        for k in ("archive_sha256", "signature_sha256", "verifier_sha256", "auth_status_sha256"): hex_string(row[k])
        for k in ("signer_primary", "signer_subkey"): hex_string(row[k], 40, True)
        require(row["signer_primary"] in ([GIT_ROOT] if source[0] == "git" else GPG_ROOTS), "signer root")
        require(row["signer_subkey"] in ([GIT_SUB] if source[0] == "git" else GPG_ROOTS), "signer subkey")
    fields(q["compiler"], "path sha256 version"); bounded_string(q["compiler"]["path"], 4096); hex_string(q["compiler"]["sha256"]); refs.append(q["compiler"]["version"])
    fields(q["sdk"], "path version settings_sha256")
    bounded_string(q["sdk"]["path"], 4096); bounded_string(q["sdk"]["version"], 128); hex_string(q["sdk"]["settings_sha256"])
    fields(q["platform"], "os_build release arch assumptions")
    for k in ("os_build", "release", "arch"): bounded_string(q["platform"][k], 128)
    checked_array(q["platform"]["assumptions"])
    for row in q["platform"]["assumptions"]:
        fields(row, "id evidence"); bounded_string(row["id"], 128); refs.append(row["evidence"])
    for key in ("patches", "configs", "logs"):
        checked_array(q[key], 8 if key == "patches" else 64); refs += q[key]
    for key in ("loaded_closure", "review", "checks"): checked_array(q[key])
    for row in q["loaded_closure"]:
        fields(row, "file kind identity assumption"); refs.append(row["file"])
        require(row["kind"] in ("private", "platform"), "closure kind"); bounded_string(row["identity"], 2048)
        if row["assumption"] is not None: bounded_string(row["assumption"], 2048)
    for row in q["review"]:
        fields(row, "id source_refs finding reason"); bounded_string(row["id"], 128); bounded_string(row["reason"], 2048)
        require(row["finding"] in ("established", "unsupported"), "review finding")
        checked_array(row["source_refs"]); refs += row["source_refs"]
    for row in q["checks"]:
        fields(row, "command exit_code source_sha256 log"); bounded_string(row["command"], 4096); checked_int(row["exit_code"], -(2**31), 2**31-1); hex_string(row["source_sha256"]); refs.append(row["log"])
    hex_string(q["command_profile_sha256"])
    require(q["candidate_status"] in ("review_ready", "unsupported"), "candidate status")
    if q["candidate_status"] == "review_ready":
        require(q["review"] and all(row["finding"] == "established" for row in q["review"]) and q["checks"] and all(row["exit_code"] == 0 for row in q["checks"]), "unsupported evidence cannot become review_ready")
    for ref in refs: ref_bytes(prefix, ref)
    return q


def prepared_budget(qualification, source_lock_bytes, component_report, basis=b"", descriptor=b""):
    require(len(qualification) <= 131072 and len(source_lock_bytes) <= 32768 and len(component_report) <= 65536
            and len(basis) <= 49152 and len(descriptor) <= 16384
            and sum(map(len, (qualification, source_lock_bytes, component_report, basis, descriptor))) <= 131072,
            "TOTAL prepared evidence exceeds 128KiB")


def file_refs(value):
    if isinstance(value, dict):
        if set(value) == {"path", "len", "sha256"}:
            yield value
        else:
            for child in value.values():
                yield from file_refs(child)
    elif isinstance(value, list):
        for child in value:
            yield from file_refs(child)


def bind_current_candidate(q, receipt, inputs, prefix, intent):
    require(q["command_profile_sha256"] == sha(canonical(command_profile())), "Commands projection currentness mismatch")
    require(q["build_id"] == intent["build_id"] == prefix.name and q["incarnation"] == intent["incarnation"], "candidate birth/incarnation mismatch")
    require(q["builder"]["source_sha256"] == inputs["source_sha256"], "builder source/recipe currentness mismatch")
    require(q["builder"]["commit"] == inputs["commit"], "builder source commit changed")
    require(q["platform"]["os_build"] == inputs["os_build"] and q["platform"]["release"] == inputs["release"] and q["platform"]["arch"] == inputs["arch"], "OS/platform currentness mismatch")
    require(receipt["os"] == {"release": inputs["release"], "arch": inputs["arch"]}, "proposal OS binding mismatch")
    require(q["compiler"]["path"] == inputs["compiler"] and q["compiler"]["sha256"] == inputs["compiler_sha256"], "compiler currentness mismatch")
    require(q["sdk"] == {"path": inputs["sdk"], "version": inputs["sdk_version"], "settings_sha256": inputs["sdk_settings_sha256"]}, "SDK currentness mismatch")
    require(q["patches"] == source_lock()["profile_patches"], "profile patch binding mismatch")
    require(q["component"]["authority_source"]["sha256"] == receipt["capability"]["probe_source_sha256"] == PROBE_SHA and q["component"]["authority_source"]["len"] == 39130, "canonical component source binding mismatch")
    require(q["component"]["probe_exe"]["sha256"] == receipt["capability"]["probe_exe_sha256"], "canonical component executable binding mismatch")
    for name in ("git", "gpg"):
        require(q["tools"][name]["path"] == "bin/" + name and receipt[name]["absolute_path"] == str(prefix / "bin" / name) and q["tools"][name]["sha256"] == receipt[name]["sha256"], "tool image/path binding mismatch")


def check_inventory(prefix, q):
    check_private_dir(prefix)
    allowed = {"qualification.json", "receipt.candidate.json"} | {ref["path"] for ref in file_refs(q)}
    licenses = strict_json(read_regular(FIX / "license-map.json", 32768), 32768)
    for component in licenses:
        for entry in component["license_files"]:
            path = "licenses/" + component["source"].rsplit("-", 1)[0] + "/" + entry["path"]
            data = read_regular(prefix / path, MAX_TAR, True)
            pinned(data, entry["sha256"], "preserved upstream license")
            allowed.add(path)
    for current, dirs, files in os.walk(prefix, followlinks=False):
        check_private_dir(Path(current))
        for name in dirs:
            check_private_dir(Path(current) / name)
        for name in files:
            path = Path(current) / name
            relative = str(path.relative_to(prefix))
            require(relative in allowed, f"unmanifested candidate file: {relative}")
            info = path.lstat()
            mode = 0o700 if relative in ("bin/git", "bin/gpg") else 0o600
            require(stat.S_ISREG(info.st_mode) and info.st_nlink == 1 and info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == mode, f"candidate file custody/mode: {relative}")


def validate_candidate(prefix, intent, inputs):
    check_private_dir(prefix)
    info = prefix.stat()
    require((info.st_dev, info.st_ino) == (intent["device"], intent["inode"]), "original prefix inode/incarnation changed")
    qbytes = read_regular(prefix / "qualification.json", 131072, True)
    receipt_bytes = read_regular(prefix / "receipt.candidate.json", 32768, True)
    q = validate_qualification(qbytes, prefix)
    receipt = validate_proposal(receipt_bytes)
    require(receipt["capability"]["evidence_sha256"] == sha(qbytes), "qualification receipt hash mismatch")
    bind_current_candidate(q, receipt, inputs, prefix, intent)
    check_inventory(prefix, q)
    return {"prefix": str(prefix), "build_id": q["build_id"], "receipt_sha256": sha(receipt_bytes), "evidence_sha256": sha(qbytes)}, q


def reuse_candidate(d, previous, run, inputs):
    intent = strict_json(read_regular(previous / "intent.json", 32768, True), 32768)
    require(intent["inputs"] == inputs, "cache recipe/platform inputs changed")
    locator, q = validate_candidate(Path(intent["prefix"]), intent, inputs)
    home = private_dir(run / "reuse-public-home")
    pinned(read_regular(VERIFIER), VERIFIER_SHA, "independent bootstrap verifier")
    # A syntactically valid cached auth record is not authentication. Repeat the
    # independent crypto check of every exact source before returning a winner.
    for source, retained in zip(PINS, q["sources"]):
        cache = d / "cache" / source[3]
        keyname = "git" if source[0] == "git" else "gnupg"
        archive, signature = cached_source(d, source, run)
        try:
            actual, _ = authenticate(source, archive, signature, read_regular(FIX / (keyname + "-release-key.asc"), MAX_ARCHIVE), home)
        except (Refused, OSError) as error:
            quarantine_cache(cache, run, source, error)
            raise
        for field in ("name", "archive_sha256", "signature_sha256", "signer_primary", "signer_subkey", "verifier_sha256", "auth_exit"):
            require(actual[field] == retained[field], "cached source authentication differs")
    exclusive(run / "reuse.json", canonical(locator))
    require(q["candidate_status"] == "review_ready", f"cached candidate remains unsupported: {locator}")
    print(canonical(locator).decode(), flush=True)
    return locator


def publish_candidate(run, prefix, q, receipt, inputs):
    # These are review proposal bytes only. There is deliberately no Root
    # admission argument, approved flag, authority-store write, or runtime grant.
    intent = strict_json(read_regular(run / "intent.json", 32768, True), 32768)
    qbytes = canonical(q)
    receipt_bytes = canonical(receipt)
    validate_qualification(qbytes, prefix)
    validate_proposal(receipt_bytes)
    require(receipt["capability"]["evidence_sha256"] == sha(qbytes), "proposal evidence mismatch before publication")
    bind_current_candidate(q, receipt, inputs, prefix, intent)
    check_inventory(prefix, q)
    publish(prefix / "qualification.json", qbytes)
    # No subsequent prefix write: the receipt is published LAST, no replacement.
    publish(prefix / "receipt.candidate.json", receipt_bytes)
    locator, _ = validate_candidate(prefix, intent, inputs)
    exclusive(run / "terminal.json", canonical({"outcome": "sealed", "locator": locator, "candidate_status": q["candidate_status"]}))
    return locator


def component(d):
    source = pinned(read_regular(AUTHORITY / "proof-helper-probe.c", 39130), PROBE_SHA, "frozen Q2 probe")
    require(len(source) == 39130, "canonical probe length")
    # Canonical source digest is a byte snapshot, never a clean-commit claim.
    digest = hashlib.sha256(b"bboxy.dream.proof-tools/source/1\0")
    for name in ("resolve.rs", "trust.rs", "c1.rs", "read.rs"):
        data = read_regular(AUTHORITY / "q2-snapshot/src/dream" / name)
        digest.update(name.encode() + b"\0" + len(data).to_bytes(8, "little") + data)
    require(digest.hexdigest() == Q2_SHA, "frozen consumer snapshot digest")
    run = session(d, "component")
    private_dir(run / "public-home"); private_dir(run / "logs")
    _, cc, _ = command(run, ["/usr/bin/xcrun", "--find", "clang"])
    _, sdk, _ = command(run, ["/usr/bin/xcrun", "--show-sdk-path"])
    _, osbuild, _ = command(run, ["/usr/bin/sw_vers", "-buildVersion"])
    compiler = cc.decode().strip()
    key = {"source": PROBE_SHA, "consumer_snapshot": Q2_SHA, "compiler": sha(read_regular(Path(compiler).resolve())), "sdk_settings": sha(read_regular(Path(sdk.decode().strip()) / "SDKSettings.json")), "os_build": osbuild.decode().strip(), "flags": ["-O2", "-Wall", "-lproc"]}
    for previous in sorted((d / "runs").iterdir()):
        path = previous / "component.json"
        if path.exists():
            record = strict_json(read_regular(path, 32768, True), 32768)
            if record["inputs"] == key:
                for name, expected in record["files"].items():
                    pinned(read_regular(previous / name, MAX_TAR, True), expected, "retained component outcome")
                print(f"retained canonical component evidence (no timing retry): {previous}", flush=True)
                return record
        elif (previous / "component-start.json").exists():
            strict_json(read_regular(previous / "component-start.json", 32768, True), 32768)
            raise Refused(f"unknown original component effect retained; inspect {previous}; changed inputs do not authorize a replacement or erase its custody")
    exclusive(run / "component-start.json", canonical(key))
    probe = run / "proof-helper-probe.c"; exe = run / "proof-helper-probe"
    exclusive(probe, source)
    command(run, [compiler, "-O2", "-Wall", "-isysroot", sdk.decode().strip(), str(probe), "-lproc", "-o", str(exe)])
    exe.chmod(0o700)
    # No second controller/schema: use the frozen program's own supported modes.
    ledger, _, _ = command(run, [str(exe), "ledger-controls"], allow_failure=True)
    root = private_dir(run / "native")
    native, _, _ = command(run, [str(exe), "native", str(root)], allow_failure=True)
    files = {str(p.relative_to(run)): sha(read_regular(p, MAX_TAR, True)) for p in (probe, exe, *sorted((run / "logs").glob("*"))) if p.is_file()}
    record = {"inputs": key, "ledger_exit": ledger, "native_exit": native, "git_exit": None,
              "files": files, "limitation": "Canonical git_capability unconditionally returns81/unproven; no real-pair component certification or Root admission. Git mode not invoked without a complete pair."}
    exclusive(run / "component.json", canonical(record))
    print(f"canonical ledger exit={ledger}, native exit={native}; retained {run}; real-pair coverage remains unproven", flush=True)
    return record


class ProvisioningTests(unittest.TestCase):
    def test_git_signature_case_refs_match_unfolded_source_bytes(self):
        data, lock = read_corpus()
        cases = corpus_signature_bindings(data, lock)
        for row in cases:
            self.assertEqual(row["extracted_len"], 228)
            self.assertEqual(row["extracted_sha256"], row["signature"]["sha256"])
            payload, signature = git_sha1_commit_parts(data[row["case"] + ".commit"])
            self.assertEqual(payload, data["valid-payload.txt"])
            self.assertEqual(signature, data[row["signature"]["path"]])
            self.assertEqual(signature.count(b"\n"), 7)
            self.assertTrue(signature.endswith(b"-----END PGP SIGNATURE-----\n"))
            self.assertNotIn(b"\r", signature)

    def test_git_signature_wrong_case_refs_fail_even_with_matching_checksums(self):
        data, lock = read_corpus()
        changed = dict(data)
        changed["valid-signature.asc"], changed["corrupt-signature.asc"] = data["corrupt-signature.asc"], data["valid-signature.asc"]
        changed_lock = json.loads(canonical(lock))
        for row in changed_lock["files"]:
            if row["path"] in ("valid-signature.asc", "corrupt-signature.asc"):
                row["len"] = len(changed[row["path"]]); row["sha256"] = sha(changed[row["path"]])
        with self.assertRaisesRegex(Refused, "signature/case Ref mismatch"):
            corpus_signature_bindings(changed, changed_lock)

    def test_git_signature_alternate_header_and_message_are_not_selected(self):
        data, _ = read_corpus()
        original = data["valid.commit"]
        payload, signature = git_sha1_commit_parts(original)
        self.assertNotIn(b"gpgsig-sha256", payload)
        self.assertNotIn(b"/NEAn0XO9RYS", signature)
        # Message bytes are not reparsed as headers. LF, spaces and CR in the
        # message remain exact, rather than normalized into another payload.
        suffix = b"gpgsig message text\n folded message\r\n"
        p, s = git_sha1_commit_parts(original + suffix)
        self.assertEqual(p, payload + suffix)
        self.assertEqual(s, signature)
        # In the deliberately changed header case, selecting the alternate
        # bytes gives a different signature, never the pinned SHA-1 Ref.
        alternate = original.replace(b"gpgsig ", b"gpgsig-ignored ", 1).replace(b"gpgsig-sha256 ", b"gpgsig ", 1)
        _, wrong = git_sha1_commit_parts(alternate)
        self.assertNotEqual(wrong, signature)

    def test_git_signature_whitespace_and_payload_substitution_refuse(self):
        data, lock = read_corpus()
        for name, replacement in (("valid-signature.asc", data["valid-signature.asc"][:-1]),
                                  ("corrupt-signature.asc", data["valid-signature.asc"]),
                                  ("valid-payload.txt", data["valid-payload.txt"] + b"\n")):
            changed = dict(data); changed[name] = replacement
            changed_lock = json.loads(canonical(lock))
            row = next(r for r in changed_lock["files"] if r["path"] == name)
            row["len"] = len(replacement); row["sha256"] = sha(replacement)
            with self.assertRaises(Refused): corpus_signature_bindings(changed, changed_lock)

    def test_git_signature_matcher_changes_commands_and_refuses_substitution(self):
        current = command_profile()
        child = next(f for f in current["forms"] if f["id"] == "verify_child")
        self.assertEqual(child["argv"], ["--keyid-format=long", "--status-fd=1", "--verify", "{git_signature_temp}", "-"])
        self.assertEqual(sum(GIT_SIGNATURE_TEMP in f["argv"] for f in current["forms"]), 1)
        for operand in ("{signature}", "valid-signature.asc", "corrupt-signature.asc", "/scratch/.git_vtag_tmpABC123"):
            stale = json.loads(canonical(current))
            next(f for f in stale["forms"] if f["id"] == "verify_child")["argv"][3] = operand
            self.assertNotEqual(sha(canonical(stale)), sha(canonical(current)))
            with self.assertRaisesRegex(Refused, "Commands projection currentness"):
                bind_current_candidate({"command_profile_sha256": sha(canonical(stale))}, {}, {}, Path("never-read-stale-prefix"), {})
            with self.assertRaisesRegex(Refused, "stale/substituted Commands"):
                capture_review(Path("never-read-stale-source"), {"commands": stale})
        prefixed = json.loads(canonical(current))
        form = next(f for f in prefixed["forms"] if f["id"] == "verify_child")
        form["argv"] = prefixed["gpg_prefix"] + form["argv"]
        with self.assertRaisesRegex(Refused, "stale/substituted Commands"):
            capture_review(Path("never-read-prefixed-source"), {"commands": prefixed})

    def test_actual_link_map_literal_bytes_do_not_change_object_table(self):
        maps = sorted((TEST_TARGET.parent / "runs").glob("*/logs/git.link-map"))
        if not maps:
            self.skipTest("requires retained real Darwin linker map")
        raw = read_regular(maps[0], 8*1024*1024, True)
        rows = parse_link_map_objects(raw, "git")
        self.assertTrue(any(b"libgit.a(" in row for row in rows))
        # Real regex literal strings in the retained map contain invalid UTF8.
        # Their bytes remain intact and never become object-path text.
        with self.assertRaises(UnicodeError): raw.decode("utf8")
        self.assertEqual(rows, parse_link_map_objects(raw + b"\xff", "git"))
        with self.assertRaises(Refused): parse_link_map_objects(raw.replace(b"# Arch: arm64", b"# Arch: x86_64", 1), "git")

    def test_public_presigned_fixture_has_real_object_relationships(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            fixture = prepare_fixture(Path(tmp))
            root = Path(fixture["repo"]["path"])
            self.assertLessEqual(len(fixture["files"]), 32)
            for ref in fixture["files"]:
                self.assertEqual(sha(read_regular(root / ref["path"], 2*1024*1024, True)), ref["sha256"])
            def obj(oid, kind):
                raw = zlib.decompress(read_regular(root / ".git/objects" / oid[:2] / oid[2:], 32768, True))
                self.assertEqual(hashlib.sha1(raw).hexdigest(), oid)
                header, body = raw.split(b"\0", 1)
                self.assertEqual(header, kind.encode() + b" " + str(len(body)).encode())
                return body
            valid = obj(fixture["valid_commit"], "commit")
            corrupt = obj(fixture["corrupt_commit"], "commit")
            ancestor = obj(fixture["ancestor"], "commit")
            self.assertNotEqual(valid, corrupt)
            self.assertIn(("parent " + fixture["ancestor"] + "\n").encode(), valid)
            tree = obj(valid.splitlines()[0].split()[1].decode(), "tree")
            self.assertIn(fixture["record_path"].encode() + b"\0" + bytes.fromhex(fixture["record_blob"]), tree)
            first_tree = obj(ancestor.splitlines()[0].split()[1].decode(), "tree")
            self.assertNotIn(fixture["record_path"].encode() + b"\0", first_tree)
            self.assertEqual(obj(fixture["record_blob"], "blob"), b"1\n")
            self.assertNotIn(b"PRIVATE KEY", read_regular(root / fixture["public_keys"]["path"]))
            # Signature validity is deliberately not inferred by this codec.

    def test_command_profile_retains_all_exact_forms_and_implicit_verify(self):
        profile = command_profile()
        self.assertEqual(len(profile["forms"]), 16)
        self.assertEqual(len({f["id"] for f in profile["forms"]}), 16)
        byid = {f["id"]: f for f in profile["forms"]}
        self.assertEqual(byid["signature"]["members"], 2)
        self.assertEqual(byid["verify_child"]["argv"], ["--keyid-format=long", "--status-fd=1", "--verify", GIT_SIGNATURE_TEMP, "-"])
        self.assertEqual(byid["last_touch"]["argv"], ["rev-list", "-1", "{commit}", "--", "{record}"])

    def test_platform_image_links_do_not_relax_private_file_custody(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            path = Path(tmp) / "fixed-platform-image"
            exclusive(path, b"public platform image bytes")
            os.link(path, Path(tmp) / "platform-alias")
            with self.assertRaises(Refused): read_regular(path)
            with self.assertRaises(Refused): read_regular(path, private=True, platform_image=True)
            self.assertEqual(read_regular(path, platform_image=True), b"public platform image bytes")

    def test_bad_cache_quarantines_its_exact_entry_without_replacement(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            d = Path(tmp); private_dir(d / "cache")
            cache = private_dir(d / "cache" / PINS[0][3])
            run = private_dir(d / "source-failure")
            exclusive(cache / "archive", b"deliberately corrupt source bytes")
            before = (cache / "archive").stat().st_ino
            with self.assertRaises(Refused): cached_source(d, PINS[0], run)
            self.assertTrue((cache / "quarantine.json").exists())
            self.assertEqual((cache / "archive").stat().st_ino, before)
            with self.assertRaisesRegex(Refused, "quarantined"):
                cached_source(d, PINS[0], run)
            self.assertEqual((cache / "archive").read_bytes(), b"deliberately corrupt source bytes")

    def test_source_snapshot_rejects_added_and_modified_native_work(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            src = Path(tmp)
            source = src / "startup.c"
            data = b"/* private source-custody witness, not executable qualification */\nint main(void){return 0;}\n"
            exclusive(source, data)
            expected = {"startup.c": {"kind": "file", "sha256": sha(data), "mode": 0o600}}
            verify_source_snapshot(src, expected)
            for added in (b"fork();", b"pthread_create(0,0,0,0);", b"start_new_service();"):
                source.write_bytes(data + added)
                with self.assertRaisesRegex(Refused, "SOURCE qualification lost"):
                    verify_source_snapshot(src, expected)
            source.write_bytes(data)
            exclusive(src / "hidden-constructor.c", b"/* unexpected source input */")
            with self.assertRaisesRegex(Refused, "SOURCE qualification lost"):
                verify_source_snapshot(src, expected)

    def test_original_unknown_build_is_retained_before_replacement(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            d = Path(tmp); private_dir(d / "runs"); private_dir(d / "candidates")
            run = private_dir(d / "runs" / "original")
            private_dir(run / "logs")
            exclusive(run / "identity.json", canonical({"kind": "build", "incarnation": 1}))
            exclusive(run / "logs/000.command.json", canonical({"effect": "unknown", "original_operation": "original"}))
            with self.assertRaisesRegex(Refused, "original build effect unresolved"):
                refuse_unknown_build_effects(d)
            self.assertEqual(sorted(p.name for p in (d / "runs").iterdir()), ["original"])
            self.assertEqual(list((d / "candidates").iterdir()), [])

    def test_failed_and_replaced_prefix_never_becomes_a_cache_winner(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            d = Path(tmp); private_dir(d / "runs"); private_dir(d / "candidates")
            run = private_dir(d / "runs" / "original")
            prefix = private_dir(d / "candidates" / "original")
            info = prefix.stat(); inputs = {"opaque_source_input": "A1"}
            exclusive(run / "intent.json", canonical({"inputs": inputs, "prefix": str(prefix), "device": info.st_dev, "inode": info.st_ino}))
            exclusive(run / "terminal.json", canonical({"outcome": "unsupported", "reason": "retained test failure"}))
            with self.assertRaisesRegex(Refused, "unchanged inputs"):
                previous_build(d, inputs, "new")
            self.assertIsNone(previous_build(d, {"opaque_source_input": "B2"}, "new"))
            prefix.rename(d / "candidates/retired-original")
            private_dir(prefix)
            with self.assertRaisesRegex(Refused, "ABA changed"):
                previous_build(d, inputs, "new")
            self.assertEqual((d / "candidates/retired-original").stat().st_ino, info.st_ino)

    def test_unknown_after_actual_link_retains_both_names_and_inode(self):
        global sync_dir
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            path = Path(tmp) / "receipt.candidate.json"
            original_sync = sync_dir
            count = 0
            def fail_after_link(parent):
                nonlocal count
                count += 1
                if count == 2:
                    raise OSError("injected parent fsync failure after actual link")
                original_sync(parent)
            sync_dir = fail_after_link
            try:
                with self.assertRaises(OSError): publish(path, b"owned exact bytes")
            finally:
                sync_dir = original_sync
            pending = list(Path(tmp).glob("*.pending-*"))
            self.assertEqual(len(pending), 1)
            self.assertEqual(path.stat().st_ino, pending[0].stat().st_ino)
            self.assertEqual(path.stat().st_nlink, 2)
            with self.assertRaises(Refused): read_regular(path)
            self.assertEqual(path.read_bytes(), b"owned exact bytes")

    def test_prepared_budget_charges_all_three_original_byte_sequences(self):
        prepared_budget(b"q" * 65536, b"s" * 32768, b"r" * 32768)
        with self.assertRaises(Refused): prepared_budget(b"q" * 65537, b"s" * 32768, b"r" * 32768)
        with self.assertRaises(Refused): prepared_budget(b"q", b"s" * 32769, b"r")
        with self.assertRaises(Refused): prepared_budget(b"q" * 65536, b"s" * 32768, b"r" * 32768, b"b", b"d")
        with self.assertRaises(Refused): prepared_budget(b"q", b"s", b"r", b"b" * 49153)
        with self.assertRaises(Refused): prepared_budget(b"q", b"s", b"r", b"b", b"d" * 16385)
        with self.assertRaises(Refused): prepared_budget(b"q", b"s", b"r" * 65537)

    def test_zero_context_patches_match_every_exact_postimage(self):
        # Upstream C includes form-feed separators. Diff line numbers must count
        # LF, not Python's wider Unicode splitlines boundaries. Verify the real
        # patch tool's output before spending a compiler build.
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            run = Path(tmp)
            private_dir(run / "src"); private_dir(run / "logs"); private_dir(run / "public-home")
            manifest = json.loads(read_regular(FIX / "patches.lock.json", 32768))
            for row in manifest:
                source = next(p for p in PINS if p[0] + "-" + p[1] == row["source"])
                cache = TEST_TARGET.parent / "cache" / source[3] / "archive"
                if not cache.exists():
                    self.skipTest("exact patch process witness requires make fetch-dream-proof-tools; no network in unit tests")
                archive = pinned(read_regular(cache, MAX_ARCHIVE, True), source[3], "test source archive")
                with tarfile.open(fileobj=io.BytesIO(tar_bytes(archive, "xz" if source[0] == "git" else "bz2")), mode="r:") as tar:
                    for entry in row["files"]:
                        dest = run / "src" / row["source"] / entry["path"]
                        dest.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
                        with tar.extractfile(row["source"] + "/" + entry["path"]) as stream:
                            exclusive(dest, pinned(stream.read(), entry["before"], "test preimage"))
            apply_profile(run)

    def test_source_key_signature_and_patch_pins(self):
        value = source_lock()
        validate_lock(canonical(value))
        for key in ("archive_sha256", "signature_sha256", "public_key", "roots", "signed_bytes"):
            changed = json.loads(canonical(value))
            changed["sources"][0][key] = "forged"
            with self.assertRaises(Refused): validate_lock(canonical(changed))
        for key in ("profile_patches", "bootstrap_verifier", "canonical_q2", "backports"):
            changed = json.loads(canonical(value)); changed[key] = ["forged"]
            with self.assertRaises(Refused): validate_lock(canonical(changed))

    def test_no_replace_preserves_original_inode_and_unknown_temp(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            root = Path(tmp); path = root / "receipt.candidate.json"
            publish(path, b"original")
            original = path.stat()
            with self.assertRaises(FileExistsError): publish(path, b"different")
            self.assertEqual(read_regular(path), b"original")
            self.assertEqual((path.stat().st_ino, path.stat().st_dev), (original.st_ino, original.st_dev))
            pending = list(root.glob("*.pending-*"))
            self.assertEqual(len(pending), 1)
            self.assertEqual(read_regular(pending[0]), b"different")

    def test_reference_custody_rejects_change_symlink_hardlink_and_traversal(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            root = Path(tmp); private_dir(root / "provenance"); private_dir(root / "provenance/blobs")
            data = b"fixed"; ref = blob_ref(data); path = root / ref["path"]
            exclusive(path, data)
            self.assertEqual(ref_bytes(root, ref), data)
            path.write_bytes(b"other")
            with self.assertRaises(Refused): ref_bytes(root, ref)
            path.unlink(); outside = root / "outside"; exclusive(outside, data)
            path.symlink_to(outside)
            with self.assertRaises(OSError): ref_bytes(root, ref)
            path.unlink(); os.link(outside, path)
            with self.assertRaises(Refused): ref_bytes(root, ref)
            with self.assertRaises(Refused): validate_ref({**ref, "path": "../outside"})
            path.unlink(); (root / "provenance/blobs").rmdir()
            (root / "provenance/blobs").symlink_to(root, target_is_directory=True)
            with self.assertRaises(OSError): ref_bytes(root, ref)

    def test_strict_json_integer_and_total_bounds(self):
        for data in (b'{"x":1,"x":1}', b'{"x": NaN}', b'{ "x":1}', b'{}\n', b'\xff', b'{"x":1.0}', b'{"x":-0}', b'{"x":"\\u0000"}', b'{"x":"\\ud800"}'):
            with self.assertRaises(Refused): strict_json(data, 128)
        with self.assertRaises(Refused): strict_json(b" " * 131073, 131072)
        for value in (-1, MAX_INT+1, True, 1.0):
            with self.assertRaises(Refused): checked_int(value)
        with self.assertRaises(Refused): checked_array([0]*65)
        with self.assertRaises(Refused): fields({"approved": True}, "schema")

    def test_secret_special_and_escaping_archive_refusal_before_effect(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            for name in ("../escape", "/absolute", "package/../../escape"):
                stream = io.BytesIO()
                with tarfile.open(fileobj=stream, mode="w") as tar:
                    entry = tarfile.TarInfo(name); entry.size = 1
                    tar.addfile(entry, io.BytesIO(b"x"))
                with self.assertRaises(Refused): extract(lzma.compress(stream.getvalue()), "xz", Path(tmp), "package")
                self.assertEqual(list(Path(tmp).iterdir()), [])


def unit(d, signature_only=False):
    global TEST_TARGET
    TEST_TARGET = private_dir(d / "unit")
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ProvisioningTests)
    if signature_only:
        suite = unittest.TestSuite(t for t in suite if t._testMethodName.startswith("test_git_signature_"))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    require(result.wasSuccessful(), "private provisioning regression failed")


def check(d, component_only=False, source_only=False, signature_corpus_only=False):
    if signature_corpus_only:
        return check_signature_corpus(d)
    if source_only:
        return check_sources(d)
    if not component_only:
        unit(d)
    result = component(d)
    require(result["ledger_exit"] == 0 and result["native_exit"] == 0, "canonical component failed; original excess/effect retained, no lucky retry")
    if component_only:
        return
    # A marker, model verdict, syntax-valid receipt or builder label cannot fill
    # Q2's absent Root-reviewed provisioning literal.
    raise Refused("No complete source/build/closure-qualified Git/GPG candidate; supplied Git probe is verification_unproven; runtime authority remains absent")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["fetch", "build", "check", "unit"])
    parser.add_argument("--target", type=Path, default=ROOT / "target")
    parser.add_argument("--component-only", action="store_true")
    parser.add_argument("--source-only", action="store_true")
    parser.add_argument("--signature-corpus-only", action="store_true")
    args = parser.parse_args()
    os.umask(0o077)
    target = args.target.absolute()
    if not target.exists():
        target.mkdir(mode=0o700, parents=True)
    d = private_dir(target / "dream-proof-tools")
    for name in ("cache", "runs", "candidates"):
        private_dir(d / name)
    FIX.mkdir(mode=0o700, parents=True, exist_ok=True)
    try:
        if args.action == "fetch":
            fetch(d)
        elif args.action == "build":
            build(d)
        elif args.action == "unit":
            unit(d)
        else:
            require(sum((args.component_only, args.source_only, args.signature_corpus_only)) <= 1, "choose one private focused check")
            check(d, args.component_only, args.source_only, args.signature_corpus_only)
    except (Refused, OSError, subprocess.TimeoutExpired) as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
