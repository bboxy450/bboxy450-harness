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

ROOT = Path(__file__).resolve().parent.parent
SCRIPT_BYTES = Path(__file__).read_bytes()
FIX = ROOT / "tests/fixtures/dream-proof-tools"
VERIFIER = "/opt/homebrew/Cellar/gnupg/2.5.24_1/bin/gpg"
VERIFIER_SHA = "21cb9adcad96b7be404014ff6e9fb837160a2135455c217dec9efe196cf0661a"
AUTHORITY = Path("/Users/feral/mydev/bboxy450-term/target/dream-proof-component-authority-7m7jv65m")
PROBE_SHA = "5493dbf0bda41ac33e4a27a9bc200f083e49501b0791d32cd255b731d6f82c33"
Q2_SHA = "55aabc5e0efd081fe050db08a40e955ddb27ff5d8a442df1c6feb911ce8160af"
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


def read_regular(path, limit=MAX_TAR, private=False):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        s = os.fstat(fd)
        require(stat.S_ISREG(s.st_mode) and s.st_nlink == 1 and s.st_size <= limit, f"unsafe/oversized regular file: {path}")
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
    except (Refused, OSError, subprocess.TimeoutExpired) as error:
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
    # One source-byte mutant in each hidden-work class must lose SOURCE input
    # qualification. This is not the separately required reachable native
    # mutant execution witness and never asserts that a census detects it.
    path = run / "src/git-2.54.0/common-main.c"
    original = read_regular(path, MAX_ARCHIVE, True)
    for name, code in [("spawn", b"fork();"), ("thread", b"pthread_create(0,0,0,0);"), ("service", b"start_new_service();")]:
        mutated = original + b"\n/* deliberately changed private source input */\n" + code + b"\n"
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
    verify_source_snapshot(run / "src", snapshot)
    exclusive(run / "source-custody.json", canonical({"source_snapshot_sha256": sha(canonical(snapshot)), "changed_source_inputs_refused": ["spawn", "thread", "service"], "runtime_lifetime": "unproven"}))
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
        tools[name] = {"path": str(path), "resolved_path": str(path.resolve()), "sha256": sha(read_regular(path.resolve()))}
    tools["python"] = {"path": sys.executable, "resolved_path": str(Path(sys.executable).resolve()), "sha256": sha(read_regular(Path(sys.executable).resolve()))}
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
    bindir = private_dir(candidate / "bin")
    exclusive(bindir / "gpg", read_regular(work / "g10/gpg"), executable=True)
    command(run, ["/usr/bin/otool", "-L", str(bindir / "gpg")])
    command(run, ["/usr/bin/otool", "-l", str(bindir / "gpg")])
    stage_linked_archives(run, candidate, deps)
    exclusive(run / "build-complete.json", canonical({"candidate": str(candidate), "tools": {name: sha(read_regular(bindir / name, private=True)) for name in ("git", "gpg")}, "lifetime": "unproven_pending_canonical_private_probe"}))
    raise Refused(f"real pair built at {candidate}; exact canonical private-candidate probe freeze still required before joined qualification")


def build_git(run, candidate, compiler, sdk, buildenv):
    # Git's release tarball is already generated, so no autotools or external
    # package discovery is used. Real supported read/signature paths remain.
    work = run / "src/git-2.54.0"
    cflags = "-O2 -g0 -isysroot " + sdk + " -DDREAM_PROOF_GPG='\"" + str(candidate / "bin/gpg") + "\"'"
    variables = ["CC=" + compiler, "CFLAGS=" + cflags, "LDFLAGS=-isysroot " + sdk + " -Wl,-map," + str(run / "logs/git.link-map"), "NO_PTHREADS=YesPlease", "NO_CURL=YesPlease", "NO_EXPAT=YesPlease", "NO_OPENSSL=YesPlease", "NO_GETTEXT=YesPlease", "NO_TCLTK=YesPlease", "NO_PERL=YesPlease", "NO_PYTHON=YesPlease", "USE_LIBPCRE=", "NO_FINK=YesPlease", "NO_DARWIN_PORTS=YesPlease", "NO_APPLE_COMMON_CRYPTO=YesPlease", "prefix=" + str(candidate)]
    command(run, ["/usr/bin/make", "-j2", *variables, "git"], cwd=work, extra_env=buildenv)
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
    for name in ("git", "gpg"):
        text = read_regular(run / "logs" / (name + ".link-map")).decode()
        cwd = run / ("src/git-2.54.0" if name == "git" else "build/gnupg/g10")
        for line in text.splitlines():
            match = re.match(r"^\[\s*\d+\]\s+(.*\.a)\([^)]*\)\s*$", line)
            if match:
                path = Path(match.group(1))
                if not path.is_absolute():
                    path = cwd / path
                path = path.resolve()
                require(path.is_relative_to(run), f"unapproved static archive path: {path}")
                archives[str(path)] = sha(read_regular(path, private=True))
    libdir = private_dir(candidate / "lib")
    summary = []
    for index, (name, digest) in enumerate(sorted(archives.items())):
        path = Path(name)
        staged_name = f"{index:02d}-" + path.name
        exclusive(libdir / staged_name, pinned(read_regular(path, private=True), digest, "linked archive"))
        status, members, _ = command(run, ["/usr/bin/ar", "-t", name])
        summary.append({"original_path": name, "staged": "lib/" + staged_name, "sha256": digest, "member_index_sha256": sha(members), "index_exit": status})
    exclusive(run / "static-closure.json", canonical(summary))


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


def prepared_budget(qualification, source_lock_bytes, component_report):
    require(len(qualification) <= 131072 and len(source_lock_bytes) <= 32768 and len(component_report) <= 131072
            and len(qualification) + len(source_lock_bytes) + len(component_report) <= 131072,
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
        for data in (b'{"x":1,"x":1}', b'{"x": NaN}', b'{ "x":1}', b'{}\n', b'\xff'):
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


def unit(d):
    global TEST_TARGET
    TEST_TARGET = private_dir(d / "unit")
    result = unittest.TextTestRunner(verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(ProvisioningTests))
    require(result.wasSuccessful(), "private provisioning regression failed")


def check(d, component_only=False, source_only=False):
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
            require(not (args.component_only and args.source_only), "choose one private focused check")
            check(d, args.component_only, args.source_only)
    except (Refused, OSError, subprocess.TimeoutExpired) as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
