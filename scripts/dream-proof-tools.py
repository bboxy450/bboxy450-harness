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
import copy
from contextlib import contextmanager, ExitStack
from unittest import mock
import struct
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
import shlex
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
# Only an explicit live Root amendment can select a FUNCTIONAL immutable input.
# The old5493 and a386 preflight checkpoints remain historical, not fallbacks.
# This is an offline ingestion premise, never a runtime approval flag or API.
# Selected by the task ORACLE FREEZE INPUT69 amendment, decisions66280/66331.
# External helpers use background QoS and total budgets; our own units retain
# the 1ms law. Custody and preparation cross-bindings precede every launch.
ROOT_Q2_INPUT = {'root': {'path': '/Users/feral/mydev/bboxy450-harness-dream-read-q2/target/dream-q2-systematic-exit-race-01M6RZZXSHZT3KHBK1QC7K3XB3/prepared-input69/dream-read-c-preparation/compile-ewu_xvzb',
          'dev': 16777234,
          'ino': 189316131,
          'uid': 501},
 'source': {'path': '/Users/feral/mydev/bboxy450-harness-dream-read-q2/target/dream-q2-systematic-exit-race-01M6RZZXSHZT3KHBK1QC7K3XB3/prepared-input69/dream-read-c-preparation/compile-ewu_xvzb/proof-helper-probe.c',
            'sha256': 'd1ec07415d4b8dcf953941f8d7f222677baf9a0d766c9fe33fb50869ea30f151',
            'ctime_ns': 1794225423978422083,
            'dev': 16777234,
            'ino': 189316132,
            'len': 432619,
            'mode': 33024,
            'mtime_ns': 1794225423978251082,
            'nlink': 1,
            'size': 432619,
            'uid': 501},
 'executable': {'path': '/Users/feral/mydev/bboxy450-harness-dream-read-q2/target/dream-q2-systematic-exit-race-01M6RZZXSHZT3KHBK1QC7K3XB3/prepared-input69/dream-read-c-preparation/compile-ewu_xvzb/probe',
                'sha256': '6c63469ff20f4a433c2fafb3a4aed7c91e037e46862da937fc316c9c140b07ba',
                'ctime_ns': 1794225425123879904,
                'dev': 16777234,
                'ino': 189316137,
                'len': 286472,
                'mode': 33088,
                'mtime_ns': 1794225425097860523,
                'nlink': 1,
                'size': 286472,
                'uid': 501},
 'preparation': {'path': '/Users/feral/mydev/bboxy450-harness-dream-read-q2/target/dream-q2-systematic-exit-race-01M6RZZXSHZT3KHBK1QC7K3XB3/prepared-input69/dream-read-c-preparation/compile-ewu_xvzb/preparation-987b8b56107a59230c0e353a7defbe1fef2ff50f7f3903a4c72a9fc290851a3a.json',
                 'sha256': '987b8b56107a59230c0e353a7defbe1fef2ff50f7f3903a4c72a9fc290851a3a',
                 'ctime_ns': 1794225425302681300,
                 'dev': 16777234,
                 'ino': 189316487,
                 'len': 532156,
                 'mode': 33024,
                 'mtime_ns': 1794225425302520424,
                 'nlink': 1,
                 'size': 532156,
                 'uid': 501},
 'source_commit': '637cb234d93d08d0c8fcfb820b7e48f9538bc0e1',
 'preparation_commit': '637cb234d93d08d0c8fcfb820b7e48f9538bc0e1',
 'signer_fingerprint': '9B07E849480C68328CA20684844ECC0587DE7AB0',
 'preparation_signer_fingerprint': '9B07E849480C68328CA20684844ECC0587DE7AB0',
 'bindings': {'build': '330fa601749351fd20b46f9cab1795f83b8aa6b1f874a888847db072acc582b2',
              'compiler': 'a3d577375b2a73419600eee3110f6bb210bbec0c10d5abd48022c9809ff23838',
              'dependency_discovery': 'be75696431139f398eee927cbe71f34db0827f5f4b36e719eaf9429962aa78e9',
              'executable': '321140749a48dcd4b30022d1f70111e1f277637e831e9f7301e83165a5e94d70',
              'header_inputs': '813697e08943a90363648618c6487406658d79713ae228dab7a6124fc740bc3d',
              'inputs': '7dfba21c503b33412eb012e0481606f69b397c595b0ff3361eb62f059c98818c',
              'linker': '7cea27482a67bfa92b546ec35ad5ba71bcedd3910bd3b12df9e618fbcfbdf01c',
              'platform': '994c24fbd511ae90c273cfcd5c5f3e78b99b70a35692800dcb48d289bdaab6e3',
              'provisioning_source': 'ba20d136e58a79da5dd4a16b2a2d923c0fe689a5390bd521407d8593f223c5dc',
              'sdk': '18544f8f94b14d8ac2da05b462679964250d8885f603422c994201d5e4e483ff',
              'input_commits': 'b2891f4bacd3884fe8647b4860185063689f5c0e856c6e884a560af8e0de9500',
              'preparation_commit': 'c3b0731a7f39586208d14f5de65780b7f2bae4823ca7fe4473d118518c17520c',
              'preparation_signer_fingerprint': '4e588b4e7396747b3b7b317d37a22c5981401da546e6eadebc4008472c4367e9',
              'preparation_snapshot': '8eefaea409f2431449344509a33b5cc1033544e7d5c9d98d0c4256cad6f7d6da',
              'retained_outputs': '734eda0fe23fd5c448e78acc8c89e55063a012b4e7276ced889212ad2b57da98',
              'retained_root': 'bd238f76e2fb713190c6c78b4778dbe2c8ab50cc408093da53f2d6cb61fe25e7',
              'retained_directory_mode': '4962ec3a0b2818a90f5ef116da5f517a840b1071e51fba8ee2bef6af2f531067'},
 'build_closure': [{'path': '/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/clang/21/lib/darwin/libclang_rt.osx.a',
                    'sha256': '5f20304aa257ed57bcba9f64287497d0be09b8cc7a5a91479b6a274294a22dfe',
                    'ctime_ns': 1790269621807429915,
                    'dev': 16777234,
                    'ino': 2620656,
                    'len': 851696,
                    'mode': 33188,
                    'mtime_ns': 1787624113000000000,
                    'nlink': 1,
                    'size': 851696,
                    'uid': 0},
                   {'path': '/Applications/Xcode.app/Contents/Developer/Toolchains/XcodeDefault.xctoolchain/usr/lib/libLTO.dylib',
                    'sha256': '08bd424518673ef9371ded084a3d614a65da784f2d898226262eb8e203801732',
                    'ctime_ns': 1790269623088330009,
                    'dev': 16777234,
                    'ino': 2621319,
                    'len': 69069424,
                    'mode': 33261,
                    'mtime_ns': 1787717325000000000,
                    'nlink': 1,
                    'size': 69069424,
                    'uid': 0}],
 'macho': {'cpu_type': 16777228,
           'cpu_subtype': 0,
           'uuid': 'e77923dd4c043a2ead94409bfef07dc6',
           'fat': False,
           'rpaths': [],
           'loads': [{'mode': 'load', 'name': '/usr/lib/libSystem.B.dylib'}]}}
Q2_PREPARATION_LIMIT = 2 * 1024 * 1024
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
    selected_q2_input()
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
    selected_q2_input()
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
    selected_q2_input()
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
    pending = [path for path in (run / "logs").glob("*.command.json")
               if not path.with_name(path.name.replace(".command.json", ".exit.json")).exists()]
    pending += [path for path in (run / "components").glob("*/spawn-intent.json")
                if not (path.parent / "wait4.json").exists()]
    return pending


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
            "os_build": osbuild.decode().strip(), "release": platform.release(), "arch": os_arch(),
            "shared_cache_uuid": current_cache_uuid()}


def current_cache_uuid():
    # Public dyld.h API, also used by the frozen canonical Q2 controller. Read
    # this process's already loaded platform; do not create a catalogue/helper.
    import ctypes
    try:
        function = ctypes.CDLL(None)._dyld_get_shared_cache_uuid
    except AttributeError as error:
        raise Refused("current platform shared-cache identity unavailable") from error
    function.argtypes = [ctypes.c_void_p]; function.restype = ctypes.c_bool
    uuid = (ctypes.c_ubyte * 16)()
    require(function(uuid), "current platform shared-cache identity unknown")
    value = bytes(uuid).hex()
    require(value != "0" * 32, "current platform shared-cache UUID is zero")
    return value


def basis_platform_current(basis, expected_uuid=None):
    bound = basis["closure"]["platform"]
    require(basis["build"]["platform"] == bound and bound["shared_cache_uuid"] == current_cache_uuid(),
            "candidate platform/shared-cache identity changed")
    if expected_uuid is not None:
        require(bound["shared_cache_uuid"] == expected_uuid, "shared-cache birth/currentness mismatch")


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
    basis = prepare_basis(run, candidate, closure)
    exclusive(run / "build-complete.json", canonical({"candidate": str(candidate), "tools": {name: sha(read_regular(bindir / name, private=True)) for name in ("git", "gpg")}, "lifetime": "unproven_pending_canonical_private_probe"}))
    return qualify_candidate(run, candidate, basis)


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
    require(0 < len(raw) <= 4096 and not any(c in raw for c in (b"\x00", b"\r", b"\n")), "link-map path bound")
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



# Shared C authority: b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9.
# The only codec shape is the signed fixture; runtime decoding remains Q2-owned.
CLOSURE_SCHEMA_SHA = "d70948ec375dae099ea1dac0b92dfb30ff87d7fcac94c98f42caa1c0ba2da8ec"
CLOSURE_VECTORS_SHA = "bff32c1afb483980952a6761918d4005c9f1e12b66f3d7aaaa88f1b9afe09740"


def closure_need(ok, code):
    require(ok, "closure:" + code)


def closure_code(error):
    reason = str(error)
    return reason[len("closure:"):] if reason.startswith("closure:") else "host_refused"


def closure_integer(text):
    closure_need(len(text) <= 21, "integer_bound")
    value = int(text)
    closure_need(-(2**63) <= value <= 2**64 - 1, "integer_bound")
    return value


def closure_no_float(_):
    raise Refused("closure:integer_grammar")


def closure_unique(pairs):
    result = {}
    for key, value in pairs:
        closure_need(key not in result, "duplicate_key")
        result[key] = value
    return result


def closure_json_decode(raw):
    try:
        return json.loads(raw.decode("utf-8", errors="strict"),
                          object_pairs_hook=closure_unique,
                          parse_int=closure_integer,
                          parse_float=closure_no_float,
                          parse_constant=closure_no_float)
    except (ValueError, UnicodeError, RecursionError) as error:
        raise Refused("closure:json_syntax") from error


def closure_walk_json(value, depth=0, maximum=10):
    # Same root0/key/value/element/scalar convention as canonical(). The private
    # authority document has its own bounded offline metadata nesting allowance.
    closure_need(depth <= maximum, "depth_bound")
    if type(value) is dict:
        for key, child in value.items():
            closure_need(type(key) is str and key.isascii() and "\x00" not in key, "key_grammar")
            closure_walk_json(key, depth + 1, maximum)
            closure_walk_json(child, depth + 1, maximum)
    elif type(value) is list:
        for child in value:
            closure_walk_json(child, depth + 1, maximum)
    elif type(value) is str:
        closure_need("\x00" not in value, "string_nul")
        try:
            value.encode("utf-8", errors="strict")
        except UnicodeError as error:
            raise Refused("closure:utf8_scalar") from error
    else:
        closure_need(value is None or type(value) in (int, bool), "integer_grammar")


def closure_strict(raw, limit=49152):
    """Stable vector causes, followed by the existing production JSON gate."""
    closure_need(type(raw) is bytes and len(raw) <= limit, "byte_bound")
    value = closure_json_decode(raw)
    closure_walk_json(value)
    closure_need(canonical(value) == raw, "canonical_bytes")
    # This remains the existing host parser; diagnostic preflight above does
    # not replace it or silently loosen any host canonicalization condition.
    closure_need(strict_json(raw, limit) == value, "host_parser_disagreement")
    return value


def closure_metadata_bytes(value):
    # Offline schema/vector metadata is not a Basis or prepared runtime bytes.
    # It embeds schema paths, contexts and malformed-wire hex as test data.
    closure_walk_json(value, maximum=32)
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=False, allow_nan=False).encode("utf-8")


def closure_authority_file(name, expected_sha256, limit):
    raw = read_regular(FIX / name, limit)
    closure_need(sha(raw) == expected_sha256, "authority_digest:" + name)
    value = closure_json_decode(raw)
    closure_need(closure_metadata_bytes(value) == raw, "authority_canonical:" + name)
    return value


def closure_authority_schema():
    return closure_authority_file("closure-boundary.schema.json", CLOSURE_SCHEMA_SHA, 32768)


def closure_schema_validate(value, rule, definitions):
    """Interpret only keywords used by the hash-pinned Schema2020-12 artifact.

    x-utf8-max-bytes is the authority's mandatory byte supplement to JSON Schema
    codepoint lengths. Cross-invariants are checked separately. This is not a
    general-purpose JSON Schema implementation or an external metaschema check.
    """
    if rule is False:
        raise Refused("closure:schema_shape")
    if rule is True:
        return
    if "$ref" in rule:
        name = rule["$ref"].removeprefix("#/$defs/")
        closure_need(rule["$ref"].startswith("#/$defs/") and name in definitions, "schema_ref")
        closure_schema_validate(value, definitions[name], definitions)
    for subrule in rule.get("allOf", []):
        closure_schema_validate(value, subrule, definitions)
    if "oneOf" in rule:
        accepted = 0
        for subrule in rule["oneOf"]:
            try:
                closure_schema_validate(value, subrule, definitions)
                accepted += 1
            except Refused:
                pass
        closure_need(accepted == 1, "schema_shape")
    if "not" in rule:
        try:
            closure_schema_validate(value, rule["not"], definitions)
        except Refused:
            pass
        else:
            raise Refused("closure:schema_shape")
    if "type" in rule:
        types = {"object": dict, "array": list, "string": str,
                 "integer": int, "null": type(None), "boolean": bool}
        closure_need(rule["type"] in types and type(value) is types[rule["type"]], "schema_shape")
    if "const" in rule:
        closure_need(canonical(value) == canonical(rule["const"]), "schema_shape")
    if "enum" in rule:
        closure_need(any(canonical(value) == canonical(choice) for choice in rule["enum"]), "schema_shape")
    if type(value) is int:
        closure_need(rule.get("minimum", value) <= value <= rule.get("maximum", value), "schema_shape")
    if type(value) is str:
        closure_need(rule.get("minLength", 0) <= len(value) <= rule.get("maxLength", len(value)), "schema_shape")
        if "pattern" in rule:
            closure_need(re.search(rule["pattern"], value) is not None, "schema_shape")
        if "x-utf8-max-bytes" in rule:
            closure_need(len(value.encode("utf-8")) <= rule["x-utf8-max-bytes"], "utf8_byte_bound")
    if type(value) is list:
        closure_need(rule.get("minItems", 0) <= len(value) <= rule.get("maxItems", len(value)), "schema_shape")
        prefix = rule.get("prefixItems", [])
        for index, item in enumerate(value):
            subrule = prefix[index] if index < len(prefix) else rule.get("items", True)
            closure_schema_validate(item, subrule, definitions)
    if type(value) is dict:
        closure_need(set(rule.get("required", [])).issubset(value), "schema_shape")
        properties = rule.get("properties", {})
        if rule.get("additionalProperties") is False:
            closure_need(set(value).issubset(properties), "schema_shape")
        for key, child in value.items():
            if key in properties:
                closure_schema_validate(child, properties[key], definitions)


def closure_unhex(value, maximum, code):
    closure_need(type(value) is str and len(value) <= 2 * maximum and len(value) % 2 == 0, code)
    closure_need(re.fullmatch("[0-9a-f]*", value) is not None, code)
    return bytes.fromhex(value)


def closure_member_frames(raw):
    """Yield exact raw name bytes; occurrences/order survive, with no64 cap.

    The bound is an offline retained-artifact limit, not a runtime allocation.
    No trim/normalization/basename conversion or UTF-8 assumption is applied.
    """
    closure_need(type(raw) is bytes and len(raw) <= 256 * 1024 * 1024, "member_frame")
    position = 0
    count = 0
    while position < len(raw):
        closure_need(position + 4 <= len(raw), "member_frame")
        size = int.from_bytes(raw[position:position + 4], "big")
        position += 4
        closure_need(1 <= size <= 4096 and position + size <= len(raw), "member_frame")
        name = raw[position:position + size]
        closure_need(not any(byte in name for byte in (b"\x00", b"\r", b"\n")), "member_name")
        count += 1
        closure_need(count <= 2**32 - 1, "member_frame")
        yield name
        position += size
    closure_need(count > 0, "member_frame")


def closure_member_summary(raw):
    count = sum(1 for _ in closure_member_frames(raw))
    # Exact u32BE(len)||raw-name concatenation. No count/domain/delimiters.
    return count, sha(raw)


def closure_member_frame(names):
    require(type(names) is list and 0 < len(names) <= 2**32 - 1, "selected member occurrence count")
    length = 0
    for name in names:
        require(type(name) is bytes and 0 < len(name) <= 4096 and not any(c in name for c in (b"\x00", b"\r", b"\n")), "selected raw member name")
        length += 4 + len(name)
        require(length <= MAX_TAR, "offline selected-member framing byte bound")
    return b"".join(len(name).to_bytes(4, "big") + name for name in names)


def closure_candidate_path(prefix, base_relative, suffix):
    # The signed authority requires resolution to remain inside the prefix.
    # Check every lexical step: a final normpath equality would allow an escape
    # followed by re-entry. Raw load bytes remain unchanged in the projection.
    closure_need(type(suffix) is str and suffix, "load_resolution")
    parts = base_relative.split("/") if base_relative else []
    for part in suffix.split("/"):
        if part in ("", "."):
            continue
        if part == "..":
            closure_need(parts, "load_resolution")
            parts.pop()
        else:
            parts.append(part)
    closure_need(parts, "load_resolution")
    return prefix + "/" + "/".join(parts)


def closure_context_validate(value, context):
    """Offline independent-evidence binding; never a Q2 runtime catalogue.

    Caller supplies freshly captured immutable image/archive/map views and raw
    referenced bytes after source/Mach-O/map/index verification. Matching claims
    authored by the candidate itself provide no authenticity or Root admission.
    None is refused rather than silently weakening the contextual checks.
    """
    names = {"prefix", "build_platform", "cache_names", "image_views", "archive_views",
             "map_views", "artifacts", "selection_views"}
    closure_need(type(context) is dict and names.issubset(context), "offline_context_required")
    prefix = context["prefix"]
    closure_need(type(prefix) is str and 0 < len(prefix.encode("utf-8")) <= 4096
                 and prefix.startswith("/") and prefix != "/"
                 and posixpath.normpath(prefix) == prefix and not prefix.startswith("//")
                 and not any(byte in prefix for byte in ("\x00", "\r", "\n")), "offline_prefix")
    closure_need(type(context["cache_names"]) is list
                 and all(type(name) is str for name in context["cache_names"]), "offline_cache_evidence")
    closure_need(value["platform"] == context["build_platform"], "platform_equality")
    images, archives = value["images"], value["statics"]
    identifiers = [image["id"] for image in images] + [archive["id"] for archive in archives]
    closure_need(len(identifiers) == len(set(identifiers)), "candidate_identity")
    closure_need(len(images) + len(archives) <= 64, "node_bound")
    candidate_files = [item["file"]["path"] for item in images + archives]
    closure_need(len(candidate_files) == len(set(candidate_files)), "candidate_identity")
    image_by_id = {image["id"]: image for image in images}
    archive_by_id = {archive["id"]: archive for archive in archives}
    other_ids = [image["id"] for image in images[2:]]
    closure_need([image["id"] for image in images[:2]] == ["git", "gpg"]
                 and other_ids == sorted(other_ids, key=lambda item: item.encode("utf-8")), "candidate_identity")
    for tool, source in (("git", "git"), ("gpg", "gnupg")):
        image = image_by_id[tool]
        closure_need(image["kind"] == "binary" and image["source"] == source
                     and image["file"]["path"] == "bin/" + tool, "candidate_identity")
    closure_need(all(image["kind"] == "dylib" for image in images[2:]), "candidate_identity")
    closure_need([archive["id"] for archive in archives]
                 == sorted(archive_by_id, key=lambda item: item.encode("utf-8")), "archive_selection")
    closure_need(sum(len(image["macho"]["loads"]) for image in images)
                 + sum(len(link_map["statics"]) for link_map in value["link_maps"]) <= 128, "edge_bound")

    reachable = {"git": {"git"}, "gpg": {"gpg"}}
    for tool in reachable:
        while True:
            previous = set(reachable[tool])
            for owner in previous:
                for load in image_by_id[owner]["macho"]["loads"]:
                    if load["target"]["kind"] == "candidate":
                        target = load["target"]["id"]
                        closure_need(target in image_by_id and image_by_id[target]["kind"] == "dylib", "load_resolution")
                        reachable[tool].add(target)
            if previous == reachable[tool]:
                break
    closure_need(set(image_by_id) == reachable["git"] | reachable["gpg"], "reachability")
    cache_names = set(context["cache_names"])
    for owner, image in image_by_id.items():
        for load in image["macho"]["loads"]:
            name = load["name"]
            closure_need(not (owner in ("git", "gpg") and name.rsplit("/", 1)[-1] == "libobjc-env.dylib"), "direct_libobjc_env")
            closure_need(not name.startswith("@rpath"), "load_resolution")
            if load["target"]["kind"] == "cache":
                closure_need(name.startswith(("/usr/lib/", "/System/Library/"))
                             and posixpath.normpath(name) == name and name in cache_names
                             and name != prefix and not name.startswith(prefix + "/"), "cache_boundary")
                continue
            target = image_by_id[load["target"]["id"]]
            expected = prefix + "/" + target["file"]["path"]
            actuals = []
            for tool in reachable:
                if owner not in reachable[tool]:
                    continue
                if name.startswith("@loader_path/"):
                    actual = closure_candidate_path(prefix, posixpath.dirname(image["file"]["path"]), name[len("@loader_path/"):])
                elif name.startswith("@executable_path/"):
                    actual = closure_candidate_path(prefix, posixpath.dirname(image_by_id[tool]["file"]["path"]), name[len("@executable_path/"):])
                else:
                    closure_need(name.startswith("/") and posixpath.normpath(name) == name, "load_resolution")
                    actual = name
                closure_need(actual.startswith(prefix + "/") and actual == expected, "load_resolution")
                actuals.append(actual)
            closure_need(actuals and len(set(actuals)) == 1, "load_resolution")

    maps = value["link_maps"]
    pairs = [(link_map["tool"], link_map["output"]) for link_map in maps]
    expected_pairs = []
    for tool in ("git", "gpg"):
        expected_pairs += [(tool, tool)] + [(tool, output) for output in sorted(reachable[tool] - {tool}, key=lambda item: item.encode("utf-8"))]
    closure_need(pairs == expected_pairs and len(set(pairs)) == len(pairs), "link_map_ownership")
    used = set()
    shared = {}
    for link_map in maps:
        output = link_map["output"]
        closure_need(link_map["output_sha256"] == image_by_id[output]["file"]["sha256"], "link_map_output")
        selected = [use["id"] for use in link_map["statics"]]
        closure_need(len(set(selected)) == len(selected) and set(selected).issubset(archive_by_id), "archive_selection")
        used.update(selected)
        invariant = (link_map["file"], link_map["output_sha256"], link_map["statics"])
        closure_need(output not in shared or shared[output] == invariant, "shared_output")
        shared[output] = invariant
    closure_need(used == set(archive_by_id), "reachability")
    closure_need(images == context["image_views"], "frozen_image_binding")
    closure_need(archives == context["archive_views"], "frozen_archive_binding")
    closure_need(maps == context["map_views"], "frozen_map_binding")

    # Bound one raw payload at a time; no persistent parallel runtime catalogue.
    # Context exists OFFLINE only and is released before component execution.
    # Ref scalars cannot validate themselves: require actual independently read
    # raw bytes here, and the upstream provenance review remains a separate gate.
    refs = [image["file"] for image in images]
    refs += [archive[key] for archive in archives for key in ("file", "member_index")]
    refs += [link_map["file"] for link_map in maps]
    required_refs = {}
    for item in refs:
        path = item["path"]
        closure_need(path not in required_refs or required_refs[path] == item, "frozen_file_binding")
        required_refs[path] = item
    artifact_rows = context["artifacts"]
    closure_need(type(artifact_rows) is list and len(artifact_rows) <= 192, "offline_artifact_bound")
    seen = set()
    for row in artifact_rows:
        closure_need(type(row) is dict and set(row) == {"path", "wire_hex"}
                     and type(row["path"]) is str and row["path"] not in seen, "offline_artifact_identity")
        seen.add(row["path"])
        if row["path"] in required_refs:
            item = required_refs[row["path"]]
            raw = closure_unhex(row["wire_hex"], 256 * 1024 * 1024, "offline_artifact_bound")
            closure_need(len(raw) == item["len"] and sha(raw) == item["sha256"], "frozen_file_binding")
    closure_need(set(required_refs).issubset(seen), "frozen_file_binding")
    selection_rows = context["selection_views"]
    closure_need(type(selection_rows) is list and len(selection_rows) <= 128, "offline_selection_bound")
    selections = {}
    for row in selection_rows:
        closure_need(type(row) is dict and set(row) == {"tool", "output", "archive", "member_frame_hex"}, "offline_selection_identity")
        key = (row["tool"], row["output"], row["archive"])
        closure_need(all(type(item) is str for item in key) and key not in selections, "offline_selection_identity")
        selections[key] = row["member_frame_hex"]
    expected_selections = set()
    for link_map in maps:
        for use in link_map["statics"]:
            key = (link_map["tool"], link_map["output"], use["id"])
            expected_selections.add(key)
            closure_need(key in selections, "offline_member_binding")
            raw = closure_unhex(selections[key], 256 * 1024 * 1024, "member_frame")
            count, digest = closure_member_summary(raw)
            closure_need(use["member_count"] == count and use["members_sha256"] == digest, "offline_member_binding")
    closure_need(set(selections) == expected_selections, "offline_member_binding")


def validate_closure(value, context=None):
    """Validate complete Closure shape and independently supplied offline facts.

    Returns value for composition; success supplies no approval, real-source
    authentication, native theorem, component result or runtime receipt. A None
    context is explicitly unsupported; there is no ambient path/cache search.
    """
    authority = closure_authority_schema()
    value = closure_strict(canonical(value))
    closure_schema_validate(value, authority, authority["$defs"])
    closure_context_validate(value, context)
    return value


def closure_budget_check(budget):
    # Synthetic actual byte buffers only. Production accounting must reserve its
    # own real bytes/copies before growth; this helper never grants a reservation.
    individual = {"basis": 49152, "descriptor": 16384, "source_lock": 32768,
                  "result": 65536, "receipt": 32768}
    closure_need(type(budget) is dict and set(budget) == {"buffers", "reserved_physical_bytes"}, "budget_shape")
    rows = budget["buffers"]
    closure_need(type(rows) is list and len(rows) <= 64, "budget_shape")
    values = {}
    physical = 0
    for row in rows:
        closure_need(type(row) is dict and set(row) == {"kind", "wire_hex", "copies"}, "budget_shape")
        kind = row["kind"]
        closure_need(type(kind) is str and kind not in values, "budget_duplicate")
        raw = closure_unhex(row["wire_hex"], 131073, "budget_individual")
        closure_need(kind in (*individual, "qualification")
                     and len(raw) <= individual.get(kind, 131072), "budget_individual")
        closure_need(type(row["copies"]) is int and 1 <= row["copies"] <= 2**64 - 1, "budget_shape")
        values[kind] = len(raw)
        physical += len(raw) * row["copies"]
    closure_need(sum(length for kind, length in values.items() if kind != "receipt") <= 131072, "budget_aggregate")
    reserved = budget["reserved_physical_bytes"]
    closure_need(type(reserved) is int and 0 <= reserved <= 2**64 - 1, "budget_shape")
    closure_need(physical <= reserved, "budget_copies")


def closure_vector_run(vector, contexts, authority):
    raw = closure_unhex(vector["wire_hex"], 49153, "vector_integrity")
    closure_need(len(raw) == vector["wire_len"] and sha(raw) == vector["wire_sha256"], "vector_integrity")
    target = vector["target"]
    try:
        if target == "member_framing":
            member_raw = closure_unhex(vector["member_frame_hex"], 256 * 1024 * 1024, "member_frame")
            closure_member_summary(member_raw)
        elif target == "byte_accounting":
            closure_budget_check(vector["budget"])
        else:
            value = closure_strict(raw)
            if target == "schema_fragment":
                definition = vector["schema_ref"]
                closure_need(definition in authority["$defs"], "vector_schema_ref")
                closure_schema_validate(value, authority["$defs"][definition], authority["$defs"])
            elif target == "closure":
                closure_schema_validate(value, authority, authority["$defs"])
                closure_need(vector["context_id"] in contexts, "vector_context")
                closure_context_validate(value, contexts[vector["context_id"]])
                closure_need(vector["publication"] == "full", "partial_publication")
                if vector["observations"] is not None:
                    observations = vector["observations"]
                    closure_need(observations["expected_incarnation"] == observations["captured_incarnation"], "stale_incarnation")
                    closure_need(all(platform == value["platform"] for platform in observations["platforms"]), "platform_observed_change")
            else:
                closure_need(target == "canonical_grammar", "vector_target")
        actual, invariant = "accept", None
    except Refused as error:
        actual, invariant = "reject", closure_code(error)
    closure_need(actual == vector["expected"], "vector_unexpected_outcome:" + vector["id"] + ":" + actual)
    if vector["invariant"] is not None:
        closure_need(invariant == vector["invariant"], "vector_wrong_cause:" + vector["id"] + ":" + str(invariant))
    return {"id": vector["id"], "actual": actual, "invariant": invariant}


def check_closure_vectors():
    """Return exact signed fixture outcomes; synthetic codec checks, no approval.

    The only IO is bounded offline reads of the two hash-pinned authority files.
    No Mach-O/Git/GPG/probe/process/network work is performed by this function.
    """
    authority = closure_authority_schema()
    corpus = closure_authority_file("closure-boundary-vectors.json", CLOSURE_VECTORS_SHA, 8 * 1024 * 1024)
    closure_need(corpus["format"] == "bboxy.dream.closure-boundary-vectors/1"
                 and corpus["schema_sha256"] == CLOSURE_SCHEMA_SHA
                 and corpus["scope"].startswith("SYNTHETIC CODEC-ONLY."), "vector_authority")
    contexts = {}
    for entry in corpus["contexts"]:
        closure_need(entry["id"] not in contexts
                     and sha(closure_metadata_bytes(entry["value"])) == entry["sha256"], "context_integrity")
        contexts[entry["id"]] = entry["value"]
    closure_need(len(corpus["vectors"]) == 324
                 and len({vector["id"] for vector in corpus["vectors"]}) == 324, "vector_ids")
    outcomes = [closure_vector_run(vector, contexts, authority) for vector in corpus["vectors"]]
    return {"count": len(outcomes), "accept": sum(row["actual"] == "accept" for row in outcomes),
            "reject": sum(row["actual"] == "reject" for row in outcomes), "outcomes": outcomes,
            "schema_sha256": CLOSURE_SCHEMA_SHA, "vectors_sha256": CLOSURE_VECTORS_SHA,
            "scope": "synthetic codec/source controls only; no native qualification or approval"}


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
            mode = {0xc: "load", 0x80000018: "weak", 0x8000001f: "reexport", 0x80000023: "upward", 0x20: "lazy"}[cmd]
            loads.append({"mode": mode, "name": string_at(command_bytes, struct.unpack_from("<I", command_bytes, 8)[0])})
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
    # Repeated commands and rpaths can be genuine. Their ordered occurrences
    # are source-bound facts, including mode, rather than deduplicated edges.
    require(len(loads) <= 64 and len(rpaths) <= 64, "oversized Mach-O load or rpath list")
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
    # Public SDK dyld.h declares this active-cache membership predicate. Only
    # direct candidate destinations are queried. No OS-image graph/catalogue
    # is prepared or read by the Q2 runtime consumer.
    try:
        contains = lib._dyld_shared_cache_contains_path
    except AttributeError as error:
        raise Refused("SDK active shared-cache membership export unavailable") from error
    contains.argtypes = [ctypes.c_char_p]
    contains.restype = ctypes.c_bool
    def bound_cache_contains(name):
        require(type(name) is str and 0 < len(name.encode()) <= 4096 and "\x00" not in name, "cache name bound")
        require(current() == snapshot, "observed shared-cache change invalidates closure")
        return bool(contains(name.encode()))
    return snapshot, bound_cache_contains, current


def closure_archive_source(run, path):
    expected = {
        "src/git-2.54.0/libgit.a": "git",
        "build/deps/lib/libgcrypt.a": "libgcrypt",
        "build/deps/lib/libassuan.a": "libassuan",
        "build/deps/lib/libnpth.a": "npth",
        "build/deps/lib/libgpg-error.a": "libgpg-error",
        "build/deps/lib/libksba.a": "libksba",
        "build/gnupg/common/libcommonpth.a": "gnupg",
        "build/gnupg/common/libgpgrl.a": "gnupg",
        "build/gnupg/kbx/libkeybox.a": "gnupg",
        "build/gnupg/regexp/libregexp.a": "gnupg",
    }
    require(path.is_relative_to(run) and path.relative_to(run).as_posix() in expected, "unreviewed linked static archive recipe path")
    return expected[path.relative_to(run).as_posix()]


def review_link_map_input(run, tool, entry, sdk, snapshot, evidence_run=None):
    """Bind every nonarchive input to this recipe or the exact trusted SDK.

    The facts stay in offline review logs; they are neither extra candidate
    graph edges nor an authorization catalogue read by the runtime consumer.
    """
    name = link_map_path(entry)
    if name.endswith(".tbd"):
        stub = Path(name)
        require(stub.is_absolute() and posixpath.normpath(name) == name and stub.is_relative_to(sdk), "foreign/nonfixed SDK stub input")
        resolved = stub.resolve()
        require(resolved.is_relative_to(sdk.resolve()), "SDK stub escapes bound SDK")
        raw = read_regular(resolved, 16 * 1024 * 1024, platform_image=True)
        # The bound SDK's v4 libSystem stub contains40 YAML documents. Its
        # first document names the linked umbrella; subsequent documents are
        # SDK reexports, not invented direct LC edges from our executable.
        require(raw.startswith(b"--- !tapi-tbd\n"), "unsupported SDK stub syntax")
        primary = raw.split(b"\n--- !tapi-tbd", 1)[0]
        require(re.search(rb"^tbd-version:\s*4\s*$", primary, re.MULTILINE), "unsupported SDK stub version")
        installs = re.findall(rb"^install-name:\s*'([^'\x00\r\n]+)'\s*$", primary, re.MULTILINE)
        require(len(installs) == 1, "unsupported/ambiguous SDK stub install name")
        install = link_map_path(installs[0])
        require(install.startswith(("/usr/lib/", "/System/Library/")) and posixpath.normpath(install) == install, "foreign SDK install name")
        facts = {"kind": "sdk_stub", "input_path": name, "path": str(resolved), "len": len(raw), "sha256": sha(raw),
                 "install_name": install, "documents": raw.count(b"--- !tapi-tbd\n")}
        if evidence_run is not None:
            directory = private_dir(evidence_run / "link-input-blobs")
            closure_retain_exact(directory / sha(raw), raw)
            facts["offline_copy"] = "link-input-blobs/" + sha(raw)
        return facts
    require(name.endswith(".o") and not name.startswith("/") and posixpath.normpath(name) == name and ".." not in name.split("/"), "unknown/foreign link-map object input")
    if tool == "git":
        require(name in ("git.o", "common-main.o") or (name.startswith("builtin/") and name.count("/") == 1), "unreviewed Git ordinary object")
        obj = run / "src/git-2.54.0" / name
        source_name = "git-2.54.0/" + name[:-2] + ".c"
    else:
        require(tool == "gpg" and "/" not in name, "unreviewed GPG ordinary object")
        obj = run / "build/gnupg/g10" / name
        source_name = "gnupg-2.5.24/g10/" + name[:-2] + ".c"
    require(source_name in snapshot and snapshot[source_name]["kind"] == "file", "ordinary object has no authenticated source mapping")
    source = pinned(read_regular(run / "src" / source_name, MAX_ARCHIVE, True), snapshot[source_name]["sha256"], "ordinary object source postimage")
    raw = read_regular(obj, MAX_TAR, True)
    return {"kind": "object", "path": str(obj), "len": len(raw), "sha256": sha(raw), "source": source_name, "source_sha256": sha(source)}


def review_link_map_inputs(run, tool, raw, sdk, evidence_run):
    snapshot = strict_json(read_regular(run / "source-snapshot.json", MAX_TAR, True), MAX_TAR)
    facts = []
    for entry in parse_link_map_objects(raw, tool):
        if re.fullmatch(rb"(.+\.a)\(([^()]*)\)", entry) is None:
            facts.append(review_link_map_input(run, tool, entry, sdk, snapshot, evidence_run))
    return facts


def selected_archive_members(run, tool):
    # The immutable raw map is authoritative. Group by first archive occurrence
    # while preserving every selected raw member name and repeated selection.
    raw = read_regular(run / "logs" / (tool + ".link-map"), 8 * 1024 * 1024, True)
    cwd = run / ("src/git-2.54.0" if tool == "git" else "build/gnupg/g10")
    selected = {}
    for entry in parse_link_map_objects(raw, tool):
        match = re.fullmatch(rb"(.+\.a)\(([^()]*)\)", entry)
        if match is None:
            require(b".a(" not in entry, "ambiguous static archive/member representation")
            continue
        path = Path(link_map_path(match[1]))
        if not path.is_absolute():
            path = cwd / path
        path = path.resolve()
        require(path.is_relative_to(run.resolve()), "foreign link-map archive")
        member = match[2]
        require(0 < len(member) <= 4096 and not any(c in member for c in (b"\x00", b"\r", b"\n")), "selected member name bound")
        selected.setdefault(str(path), []).append(member)
    return raw, selected


def emit_closure(run, prefix, *, source_run=None, evidence_prefix=None):
    """Emit the single signed C projection, without running either candidate.

    All candidate images/selected static inputs stay explicit. Platform internals
    are the named OS/cache trust premise. Raw evidence remains offline; its Ref
    scalars are charged in prepared bytes. An unknown private image refuses.
    """
    source_run = run if source_run is None else source_run
    evidence_prefix = prefix if evidence_prefix is None else evidence_prefix
    meta = strict_json(read_regular(source_run / "platform.json", 32768, True), 32768)
    intent = strict_json(read_regular(source_run / "intent.json", 32768, True), 32768)
    check_private_dir(prefix)
    check_private_dir(prefix / "bin")
    check_private_dir(prefix / "lib")
    identity = prefix.stat()
    require(str(prefix) == intent["prefix"] and (identity.st_dev, identity.st_ino) == (intent["device"], intent["inode"]), "original prefix effect/incarnation changed before projection")
    snapshot, contains, current_cache = capture_dyld_context(run, evidence_prefix, Path(meta["sdk"]))
    _, os_output, _ = command(run, ["/usr/bin/sw_vers", "-buildVersion"])
    os_build = os_output.decode("ascii").strip()
    release, machine = platform.release(), platform.machine()
    require(re.fullmatch(r"[0-9]{2}[A-Z][0-9]{1,6}[a-z]?", os_build) and re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,2}", release), "unknown OS build/release form")
    require(machine in ("arm64", "aarch64") and os_arch() == "aarch64", "unsupported platform architecture")
    require((os_build, release, os_arch()) == (intent["inputs"]["os_build"], intent["inputs"]["release"], intent["inputs"]["arch"]), "retained build OS tuple changed")
    platform_binding = {"arch": "aarch64", "darwin_machine": machine, "release": release, "os_build": os_build,
                        "shared_cache_uuid": snapshot["shared_cache_uuid"], "premise": "trusted_platform_boundary/1"}
    records = strict_json(read_regular(source_run / "static-closure.json", 1024 * 1024, True), 1024 * 1024)
    require(type(records) is list and len(records) <= 62, "candidate/static node bound")
    images, archives, maps, selections = [], [], [], []
    image_identities = {}
    archive_by_path = {}
    # This transient validation context is owned by the offline producer, never
    # serialized as prepared evidence or consulted as runtime authorization.
    artifacts = {}
    def remember(ref, raw):
        require(len(raw) == ref["len"] and sha(raw) == ref["sha256"], "offline Closure Ref binding")
        require(ref["path"] not in artifacts or artifacts[ref["path"]] == raw, "conflicting offline Ref")
        artifacts[ref["path"]] = raw
        return ref
    for row in records:
        path = Path(row["original_path"])
        require(path.is_absolute() and path.is_relative_to(source_run.resolve()), "foreign retained archive path")
        require(str(path) not in archive_by_path, "duplicate original static archive")
        validate_ref({"path": row["staged"], "len": 0, "sha256": row["sha256"]})
        raw = pinned(read_regular(prefix / row["staged"], MAX_TAR, True), row["sha256"], "staged linked archive")
        require(sha(read_regular(path, MAX_TAR, True)) == row["sha256"], "original archive bytes changed")
        ref = remember({"path": row["staged"], "len": len(raw), "sha256": sha(raw)}, raw)
        require(ref_bytes(prefix, ref) == raw, "archive no-follow currentness")
        index_raw = ref_bytes(prefix, row["member_index"], 1024 * 1024)
        require(sha(index_raw) == row["member_index_sha256"] and index_raw.endswith(b"\n") and b"\r" not in index_raw and b"\x00" not in index_raw, "archive index representation/binding")
        index_names = index_raw[:-1].split(b"\n")
        require(index_names and all(0 < len(n) <= 4096 for n in index_names), "archive index member bound")
        remember(row["member_index"], index_raw)
        node_id = "static-" + path.name.removesuffix(".a")
        archive = {"id": node_id, "source": closure_archive_source(source_run, path), "file": ref, "member_index": row["member_index"]}
        archives.append(archive)
        archive_by_path[str(path)] = (archive, row, index_names)
    cache_names = []
    for tool in ("git", "gpg"):
        raw = read_regular(prefix / "bin" / tool, MAX_TAR, True)
        require(stat.S_IMODE((prefix / "bin" / tool).lstat().st_mode) == 0o700, "candidate executable custody")
        built = source_run / ("src/git-2.54.0/git" if tool == "git" else "build/gnupg/g10/gpg")
        require(read_regular(built, MAX_TAR, True) == raw, "staged image differs from actual linked output")
        image_identities[tool] = absolute_identity(prefix / "bin" / tool)
        require(image_identities[tool]["sha256"] == sha(raw), "image changed during initial identity capture")
        image_ref = remember({"path": "bin/" + tool, "len": len(raw), "sha256": sha(raw)}, raw)
        require(ref_bytes(prefix, image_ref) == raw, "image no-follow currentness")
        macho, install, _ = parse_macho(raw, binary=True)
        require(install is None, "candidate binary install name")
        loads = []
        for load in macho["loads"]:
            name = load["name"]
            require(name.rsplit("/", 1)[-1] != "libobjc-env.dylib", "direct root libobjc-env is Unsupported")
            # This all-static recipe has no reviewed private dylib. It must
            # refuse an unexpected candidate/foreign/@rpath destination, never
            # drop it or describe a root-only approximation as complete.
            require(name.startswith(("/usr/lib/", "/System/Library/")) and posixpath.normpath(name) == name,
                    "unexpected private/foreign/unresolved image requires exact source/link review")
            require(name != str(prefix) and not name.startswith(str(prefix) + "/"), "candidate-prefix image cannot use the trusted cache boundary")
            require(contains(name), "noncache candidate destination is Unsupported")
            cache_names.append(name)
            loads.append({**load, "target": {"kind": "cache"}})
        macho = {**macho, "loads": loads}
        images.append({"id": tool, "kind": "binary", "source": "git" if tool == "git" else "gnupg", "file": image_ref,
                       "macho": macho, "install_name": None})
        map_raw, selected = selected_archive_members(source_run, tool)
        facts = review_link_map_inputs(source_run, tool, map_raw, Path(meta["sdk"]), run)
        closure_retain_exact(run / (tool + "-link-input-review.json"), canonical(facts))
        map_ref = remember(retain_blob(evidence_prefix, map_raw), map_raw)
        uses = []
        for path, names in selected.items():
            require(path in archive_by_path, "map uses an unstaged/unreviewed archive")
            archive, retained, index_names = archive_by_path[path]
            require(all(n in index_names for n in names), "selected member absent from actual archive index")
            require([link_map_path(n) for n in names] == retained["selected_members"].get(tool), "retained selection differs from actual map order/multiplicity")
            frame = closure_member_frame(names)
            uses.append({"id": archive["id"], "member_count": len(names), "members_sha256": sha(frame)})
            selections.append({"tool": tool, "output": tool, "archive": archive["id"], "member_frame_hex": frame.hex()})
        maps.append({"tool": tool, "output": tool, "file": map_ref, "output_sha256": image_ref["sha256"], "statics": uses})
    archives.sort(key=lambda a: a["id"].encode())
    closure = {"roots": ["git", "gpg"], "images": images, "statics": archives, "link_maps": maps, "platform": platform_binding}
    context = {"prefix": str(prefix), "build_platform": platform_binding, "cache_names": sorted(set(cache_names)),
               "image_views": images, "archive_views": archives, "map_views": maps, "selection_views": selections,
               "artifacts": [{"path": path, "wire_hex": raw.hex()} for path, raw in artifacts.items()]}
    validate_closure(closure, context)
    for tool, identity_before in image_identities.items():
        require(absolute_identity(prefix / "bin" / tool) == identity_before, "observed image identity change invalidates projection")
    # A mismatch is terminal for this original operation; matching a later A
    # cannot erase observed B. Equal endpoints do not prove unseen changes.
    require(current_cache() == snapshot and (prefix.stat().st_dev, prefix.stat().st_ino) == (identity.st_dev, identity.st_ino), "observed platform/prefix change invalidates original projection")
    closure_retain_exact(run / "closure.json", canonical(closure))
    closure_retain_exact(run / "closure-boundary-observations.json", canonical({"platform": platform_binding,
                         "cache_names": sorted(set(cache_names)), "sdk_context": "dyld-native-context.json",
                         "premise": "trusted_platform_boundary/1; checkpoint observations only; no unobserved-change/native theorem"}))
    return closure


def check_closure_projection(d):
    # Explicit task authority permits a C projection of this already-retained
    # authenticated run. Its failed build and candidate are never modified.
    source_run = SIGNATURE_SOURCE_RUN
    run = session(d, "closure-projection")
    private_dir(run / "logs")
    private_dir(run / "public-home")
    evidence = private_dir(run / "projection-provenance")
    before = read_regular(source_run / "intent.json", 32768, True)
    closure = emit_closure(run, Path(strict_json(before, 32768)["prefix"]), source_run=source_run, evidence_prefix=evidence)
    require(read_regular(source_run / "intent.json", 32768, True) == before, "retained build identity changed")
    exclusive(run / "projection-incomplete.json", canonical({"intended_basis_format": "bboxy.dream.private-pretest-basis/1",
              "scope": "Closure projection ONLY from retained authenticated metadata; incomplete/unapproved, no Basis/probe/qualification/receipt",
              "retained_build_id": source_run.name, "retained_intent_sha256": sha(before),
              "closure_sha256": sha(canonical(closure)), "closure_bytes": len(canonical(closure)),
              "schema_sha256": CLOSURE_SCHEMA_SHA, "vectors_sha256": CLOSURE_VECTORS_SHA,
              "runtime_approved": False, "full_basis_fit": "unknown", "component": "not_run"}))
    print(f"incomplete C Closure projection retained at {run}; no native/component qualification", flush=True)



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
    q2_input = selected_q2_input()
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
    retain_blob(prefix, q2_input["source_bytes"])
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
    # The descriptor does not exist yet. Charge the materialized Basis/lock
    # plus the bounded future result; private_descriptor charges its ACTUAL
    # bytes before launch. Summing both individual ceilings rejected fitting
    # inputs (47112+7176+65536+16384) before their descriptor was constructed.
    # The final envelope is charged with the retained actual result before
    # publication. Every phase keeps the same individual and aggregate caps.
    prepared_budget(b"", ref_bytes(prefix, lock), b"\0" * 65536, raw)
    exclusive(prefix / "provenance/pretest-basis.json", raw)
    exclusive(run / "basis-ready.json", canonical({"basis": file_pin(prefix, "provenance/pretest-basis.json"), "source_lock": file_pin(prefix, lock["path"]), "probe_admitted": False}))
    return basis


# Offline Q2 input ingestion. The selected receipt is build provenance, not a
# runtime Basis, a second controller, or an uncharged prepared input catalogue.
Q2_ID_FIELDS = "path sha256 ctime_ns dev ino len mode mtime_ns nlink size uid"
Q2_RECEIPT_FIELDS = "basis build candidate_run compiler created dependency_discovery executable format header_inputs input_commits inputs limitations linker platform preparation_commit preparation_signature_verified preparation_signer_fingerprint preparation_snapshot private_test provisioning_source retained_directory_mode retained_outputs retained_root runtime_approved schema_authority_commit sdk signature_verified signer_fingerprint source_commit source_snapshot whole_command_lifetime_proof"
Q2_BINDINGS = "build compiler dependency_discovery executable header_inputs inputs linker platform provisioning_source sdk input_commits preparation_commit preparation_signer_fingerprint preparation_snapshot retained_outputs retained_root retained_directory_mode"
Q2_PREPARATION_NAMES = ("Makefile", "tests/fixtures/dream-read/prepare-c-probe.py", "tests/fixtures/dream-read/check-c-preparation.py")
Q2_SOURCE_NAMES = ("src/dream/c1.rs", "src/dream/read.rs", "src/dream/resolve.rs", "src/dream/trust.rs",
                  "tests/fixtures/dream-proof-tools/closure-boundary-vectors.json", "tests/fixtures/dream-proof-tools/closure-boundary.schema.json",
                  "tests/fixtures/dream-read/proof-helper-probe.c")
Q2_INPUT_NAMES = (*Q2_PREPARATION_NAMES, *Q2_SOURCE_NAMES)
Q2_C_SOURCE = "tests/fixtures/dream-read/proof-helper-probe.c"


def q2_identity_shape(pin):
    fields(pin, Q2_ID_FIELDS)
    bounded_string(pin["path"], 4096); hex_string(pin["sha256"])
    for key in Q2_ID_FIELDS.split():
        if key not in ("path", "sha256"):
            checked_int(pin[key], -(2**63) if key in ("mtime_ns", "ctime_ns") else 0, 2**64 - 1)
    require(pin["size"] == pin["len"] and pin["nlink"] == 1, "Q2 selected identity shape")


def q2_metadata_hash(value):
    # Offline receipt/header arrays may exceed prepared runtime limits. Only
    # bounded offline metadata uses this codec; it is never passed to Q2.
    closure_walk_json(value, maximum=32)
    return sha(json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode())


def q2_fd_snapshot(fd, path, limit, *, private, executable=False, collect=False):
    before = os.fstat(fd)
    require(stat.S_ISREG(before.st_mode) and before.st_nlink == 1 and 0 <= before.st_size <= limit, "Q2 regular one-link file/size")
    if private:
        require(before.st_uid == os.getuid() and stat.S_IMODE(before.st_mode) in ((0o500, 0o700) if executable else (0o400, 0o600)), "Q2 private file custody/mode")
    digest = hashlib.sha256(); data = bytearray(); length = 0
    while True:
        raw = os.read(fd, min(65536, limit + 1 - length))
        if not raw: break
        length += len(raw); require(length <= limit, "Q2 file growth bound")
        digest.update(raw)
        if collect: data.extend(raw)
    after = os.fstat(fd)
    attrs = ("st_dev", "st_ino", "st_size", "st_mode", "st_uid", "st_nlink", "st_mtime_ns", "st_ctime_ns")
    require(length == before.st_size and all(getattr(before, k) == getattr(after, k) for k in attrs), "Q2 input changed during held read")
    identity = {"path": str(path), "sha256": digest.hexdigest(), "ctime_ns": before.st_ctime_ns,
                "dev": before.st_dev, "ino": before.st_ino, "len": length, "mode": before.st_mode,
                "mtime_ns": before.st_mtime_ns, "nlink": before.st_nlink, "size": before.st_size, "uid": before.st_uid}
    return identity, bytes(data) if collect else None


def q2_private_file(root, path, *, expected=None, executable=False, limit=MAX_ARCHIVE, collect=False):
    # All private components are walked relative to the held selected root.
    # No leaf or ancestor symlink may redirect an immutable selected input.
    fields(root, "path dev ino uid")
    bounded_string(root["path"], 4096)
    if expected is not None:
        q2_identity_shape(expected)
        require(expected["len"] <= limit, "Q2 selected input exceeds offline byte limit")
    base = Path(root["path"]); path = Path(path)
    require(base.is_absolute() and base == base.resolve() and str(base) == posixpath.normpath(str(base)) and base != Path("/"), "Q2 fixed private root")
    for key in ("dev", "ino", "uid"): checked_int(root[key], 0, 2**64 - 1)
    require(path.is_absolute() and path.is_relative_to(base) and str(path) == posixpath.normpath(str(path)), "Q2 input outside selected root")
    relative = path.relative_to(base).parts
    require(relative and all(part not in ("", ".", "..") for part in relative), "Q2 fixed relative input")
    fd = os.open(base, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        info = os.fstat(fd)
        require((info.st_dev, info.st_ino, info.st_uid) == (root["dev"], root["ino"], root["uid"]) and info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700, "Q2 selected root identity/custody")
        for part in relative[:-1]:
            child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
            os.close(fd); fd = child
            info = os.fstat(fd)
            require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700, "Q2 intermediate directory custody")
        leaf = os.open(relative[-1], os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK, dir_fd=fd)
        try:
            actual, data = q2_fd_snapshot(leaf, path, limit, private=True, executable=executable, collect=collect)
            named = os.stat(relative[-1], dir_fd=fd, follow_symlinks=False)
            require((named.st_dev, named.st_ino, named.st_ctime_ns) == (actual["dev"], actual["ino"], actual["ctime_ns"]), "Q2 named input identity changed after read")
            if expected is not None:
                q2_identity_shape(expected)
                require(actual == expected, "Q2 selected file identity/bytes mismatch")
            return actual, data
        finally: os.close(leaf)
    finally: os.close(fd)


def q2_platform_file(expected, roots):
    q2_identity_shape(expected)
    path = Path(expected["path"])
    require(path.is_absolute() and str(path) == posixpath.normpath(str(path)), "Q2 fixed platform input")
    # SDK root aliases are explicit selected platform inputs. Resolve only
    # inside one of those roots, then hold a no-follow leaf and compare identity.
    require(any(path.is_relative_to(root) and path.resolve().is_relative_to(root.resolve()) for root in roots), "Q2 foreign compiler/SDK/header/closure input")
    fd = os.open(path.resolve(), os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC | os.O_NONBLOCK)
    try:
        actual, _ = q2_fd_snapshot(fd, path, MAX_TAR, private=False)
        require(actual == expected, "Q2 compiler/SDK/header/closure identity changed")
        named = path.stat()
        require((named.st_dev, named.st_ino, named.st_ctime_ns) == (actual["dev"], actual["ino"], actual["ctime_ns"]), "Q2 platform input path changed after read")
    finally: os.close(fd)


def q2_private_directory(root, expected):
    fields(expected, "kind path dev ino uid mode")
    require(expected["kind"] == "directory", "Q2 retained directory kind")
    base, path = Path(root["path"]), Path(expected["path"])
    require(base.is_absolute() and base == base.resolve() and path.is_relative_to(base)
            and str(path) == posixpath.normpath(str(path)), "Q2 retained directory outside selected root")
    fd = os.open(base, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        for part in (None, *path.relative_to(base).parts):
            if part is not None:
                child = os.open(part, os.O_RDONLY | os.O_DIRECTORY | os.O_NOFOLLOW | os.O_CLOEXEC, dir_fd=fd)
                os.close(fd); fd = child
            info = os.fstat(fd)
            require(info.st_uid == os.getuid() and stat.S_IMODE(info.st_mode) == 0o700, "Q2 retained directory custody")
            if part is None:
                require((info.st_dev, info.st_ino, info.st_uid) == (root["dev"], root["ino"], root["uid"]), "Q2 retained root identity")
        actual = {"kind": "directory", "path": str(path), "dev": info.st_dev, "ino": info.st_ino,
                  "uid": info.st_uid, "mode": info.st_mode}
        require(actual == expected, "Q2 retained directory identity changed")
    finally: os.close(fd)


def q2_header_file(root, pin, roots):
    # The compiler's lexical include name, every symlink identity/target byte,
    # and the final regular header are separate bound inputs. No alias collapse.
    fields(pin, Q2_ID_FIELDS + " resolved_path snapshot symlinks")
    identity = {key: pin[key] for key in Q2_ID_FIELDS.split()}
    q2_identity_shape(identity)
    lexical = Path(pin["path"])
    require(lexical.is_absolute() and str(lexical) == posixpath.normpath(str(lexical))
            and any(lexical.is_relative_to(base) for base in roots), "Q2 foreign lexical header")
    links = pin["symlinks"]
    require(type(links) is list and len(links) <= 40, "Q2 lexical symlink bound")
    pending = list(lexical.parts[1:]); current = Path("/"); observed_links = []; copies = []
    steps = 0
    while pending:
        steps += 1; require(steps <= 4096, "Q2 lexical header traversal bound")
        part = pending.pop(0)
        if part == "..": current = current.parent; continue
        if part == ".": continue
        current /= part
        before = current.lstat()
        if stat.S_ISLNK(before.st_mode):
            require(len(observed_links) < len(links), "Q2 unbound header symlink")
            expected = links[len(observed_links)]
            fields(expected, Q2_ID_FIELDS + " snapshot target")
            q2_identity_shape({key: expected[key] for key in Q2_ID_FIELDS.split()})
            target = os.readlink(current); raw = os.fsencode(target); after = current.lstat()
            attrs = ("st_dev", "st_ino", "st_size", "st_mode", "st_uid", "st_nlink", "st_mtime_ns", "st_ctime_ns")
            require(all(getattr(before, key) == getattr(after, key) for key in attrs), "Q2 header symlink changed during read")
            actual = {"path": str(current), "sha256": sha(raw), "ctime_ns": before.st_ctime_ns,
                      "dev": before.st_dev, "ino": before.st_ino, "len": len(raw), "mode": before.st_mode,
                      "mtime_ns": before.st_mtime_ns, "nlink": before.st_nlink, "size": before.st_size,
                      "uid": before.st_uid, "target": target}
            require(actual == {key: value for key, value in expected.items() if key != "snapshot"}, "Q2 header symlink identity/target mismatch")
            copied, wire = q2_private_file(root, expected["snapshot"], collect=True)
            require(wire == raw, "Q2 retained symlink target bytes")
            copies.append(copied); observed_links.append(actual)
            replacement = Path(target)
            require(target and len(raw) <= 4096 and "\x00" not in target, "Q2 header symlink target bound")
            if replacement.is_absolute(): current = Path("/"); pending = list(replacement.parts[1:]) + pending
            else: current = current.parent; pending = list(replacement.parts) + pending
        elif pending:
            require(stat.S_ISDIR(before.st_mode), "Q2 lexical header non-directory")
    require(len(observed_links) == len(links) and str(current) == pin["resolved_path"], "Q2 lexical header resolved path/symlink coverage")
    q2_platform_file({**identity, "path": str(current)}, roots)
    copied, _ = q2_private_file(root, pin["snapshot"])
    require((copied["len"], copied["sha256"]) == (pin["len"], pin["sha256"]), "Q2 retained header bytes")
    copies.append(copied)
    return copies


def q2_retained_outputs(root, rows, required):
    require(type(rows) is list and 0 < len(rows) <= 8192, "Q2 retained output bound")
    paths = []
    for row in rows:
        require(type(row) is dict and row.get("kind") in ("file", "directory"), "Q2 retained output kind")
        if row["kind"] == "directory":
            q2_private_directory(root, row)
        else:
            fields(row, Q2_ID_FIELDS + " kind")
            identity = {key: row[key] for key in Q2_ID_FIELDS.split()}
            require(stat.S_IMODE(identity["mode"]) in (0o400, 0o500), "Q2 retained output is not immutable")
            q2_private_file(root, identity["path"], expected=identity, executable=bool(identity["mode"] & 0o100))
        paths.append(row["path"])
    require(len(paths) == len(set(paths)) and paths == sorted(paths), "Q2 retained output order/duplicate")
    expected_paths = {root["path"]}
    for name in required:
        path = Path(name)
        require(path.is_relative_to(Path(root["path"])), "Q2 required retained output outside root")
        expected_paths.add(str(path))
        expected_paths.update(str(parent) for parent in path.parents if parent.is_relative_to(Path(root["path"])))
    require(set(paths) == expected_paths, "Q2 retained output coverage differs from actual referenced inputs")


def q2_current_platform():
    import plistlib
    raw = read_regular("/System/Library/CoreServices/SystemVersion.plist", 65536, platform_image=True)
    return {"machine": platform.machine(), "release": platform.release(), "os_build": plistlib.loads(raw)["ProductBuildVersion"]}


def ingest_q2_input(selection, *, current_platform=None):
    """Read and validate a Root-selected immutable input WITHOUT executing it.

    The explicit selection is an internal unit-test seam, not a CLI/env option.
    Production uses selected_q2_input() only. Success is offline ingestion, never
    source/native qualification, permission to launch, or runtime approval.
    """
    try:
        return _ingest_q2_input(selection, current_platform=current_platform)
    except (KeyError, TypeError, AttributeError, IndexError, ValueError, UnicodeError) as error:
        raise Refused("malformed selected Q2 offline input: " + type(error).__name__) from error


def _ingest_q2_input(selection, *, current_platform=None):
    require(selection is not None, "functional Q2 input is not selected by Root; historical preflights remain stopped")
    fields(selection, "root source executable preparation source_commit signer_fingerprint preparation_commit preparation_signer_fingerprint bindings build_closure macho")
    root = selection["root"]
    for name in ("source", "executable", "preparation"): q2_identity_shape(selection[name])
    for name in ("source_commit", "preparation_commit"): hex_string(selection[name], 40)
    for name in ("signer_fingerprint", "preparation_signer_fingerprint"):
        hex_string(selection[name], 40, True)
        require(selection[name] == "9B07E849480C68328CA20684844ECC0587DE7AB0", "Q2 selected signer is not the approved Boxy identity")
    fields(selection["bindings"], Q2_BINDINGS)
    for digest in selection["bindings"].values(): hex_string(digest)
    source, source_bytes = q2_private_file(root, selection["source"]["path"], expected=selection["source"], collect=True)
    exe, exe_bytes = q2_private_file(root, selection["executable"]["path"], expected=selection["executable"], executable=True, collect=True)
    receipt_pin, raw = q2_private_file(root, selection["preparation"]["path"], expected=selection["preparation"], limit=Q2_PREPARATION_LIMIT, collect=True)
    # This is the preparation producer's compact sorted JSON+LF, deliberately
    # distinct from the no-LF/depth10 prepared runtime grammar.
    receipt = closure_json_decode(raw)
    closure_walk_json(receipt, maximum=32)
    encoded = json.dumps(receipt, sort_keys=True, separators=(",", ":"), ensure_ascii=False, allow_nan=False).encode() + b"\n"
    require(encoded == raw, "Q2 offline preparation encoding")
    fields(receipt, Q2_RECEIPT_FIELDS)
    private_observed = []
    platform_observed = []
    require(receipt["format"] == "bboxy.dream.c-preparation/1" and receipt["basis"] == "conditional_source_review"
            and receipt["private_test"] == "not_applicable_private_test" and receipt["candidate_run"] is False
            and receipt["runtime_approved"] is False and receipt["whole_command_lifetime_proof"] is False, "Q2 preparation scope is not runtime authority")
    require(receipt["source_commit"] == selection["source_commit"] and receipt["signer_fingerprint"] == selection["signer_fingerprint"]
            and receipt["signature_verified"] is True, "Q2 selected signed provenance mismatch")
    require(receipt["preparation_commit"] == selection["preparation_commit"]
            and receipt["preparation_signer_fingerprint"] == selection["preparation_signer_fingerprint"]
            and receipt["preparation_signature_verified"] is True, "Q2 selected signed preparation provenance mismatch")
    # signature_verified is only a receipt assertion. Authenticity is the
    # independently Root-selected signed commit and exact receipt/source bytes.
    for name, digest in selection["bindings"].items():
        require(q2_metadata_hash(receipt[name]) == digest, "Q2 selected preparation projection mismatch: " + name)
    require(receipt["executable"] == exe, "Q2 supplied executable/receipt cross-binding")
    require(receipt["retained_root"] == root["path"] and receipt["retained_directory_mode"] == "0700", "Q2 retained root/custody binding")
    bounded_string(receipt["source_snapshot"], 4096)
    bounded_string(receipt["preparation_snapshot"], 4096)
    snapshot = Path(receipt["source_snapshot"])
    preparation_snapshot = Path(receipt["preparation_snapshot"])
    require(snapshot.is_relative_to(Path(root["path"])), "Q2 source snapshot root binding")
    # Root may freeze the original compiler input instead of its signed-source
    # snapshot copy. Both retained files must still match the signed bytes;
    # only the exact root-level compiler input identity is an alternative.
    require(source["path"] == str(snapshot / Q2_C_SOURCE)
            or (source["path"] == str(Path(root["path"]) / "proof-helper-probe.c")
                and source == receipt["build"]["input"]), "Q2 source snapshot/compiler input binding")
    require(preparation_snapshot.is_relative_to(Path(root["path"])) and preparation_snapshot != snapshot,
            "Q2 distinct preparation snapshot path binding")
    fields(receipt["inputs"], " ".join(Q2_INPUT_NAMES))
    fields(receipt["input_commits"], " ".join(Q2_INPUT_NAMES))
    snapshot_data = {}
    for name, ref in receipt["inputs"].items():
        fields(ref, "len sha256"); checked_int(ref["len"], 0, MAX_ARCHIVE); hex_string(ref["sha256"])
        preparation_input = name in Q2_PREPARATION_NAMES
        require(receipt["input_commits"][name] == selection["preparation_commit" if preparation_input else "source_commit"],
                "Q2 source/preparation input commit binding")
        actual, content = q2_private_file(root, (preparation_snapshot if preparation_input else snapshot) / name, limit=MAX_ARCHIVE, collect=True)
        require(actual["len"] == ref["len"] and actual["sha256"] == ref["sha256"], "Q2 signed source snapshot bytes changed")
        snapshot_data[name] = content
        private_observed.append(actual)
    require(receipt["inputs"][Q2_C_SOURCE] == {"len": source["len"], "sha256": source["sha256"]}, "Q2 exact selected source length/hash")
    require(snapshot_data[Q2_C_SOURCE] == source_bytes, "Q2 selected source differs from compiler snapshot")
    for name, digest in (("closure-boundary.schema.json", CLOSURE_SCHEMA_SHA), ("closure-boundary-vectors.json", CLOSURE_VECTORS_SHA)):
        require(receipt["inputs"]["tests/fixtures/dream-proof-tools/" + name]["sha256"] == digest, "Q2 shared C authority mismatch")
    require(receipt["schema_authority_commit"] == "b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9", "Q2 C authority provenance")
    digest = hashlib.sha256(b"bboxy.dream.proof-tools/source/1\0")
    for name in ("resolve.rs", "trust.rs", "c1.rs", "read.rs"):
        content = snapshot_data["src/dream/" + name]
        digest.update(name.encode() + b"\0" + len(content).to_bytes(8, "little") + content)
    fields(receipt["provisioning_source"], "compiled_product_currentness_proven domain kind sha256")
    require(receipt["provisioning_source"] == {"compiled_product_currentness_proven": False, "domain": "bboxy.dream.proof-tools/source/1\\0",
            "kind": "exact_source_snapshot", "sha256": digest.hexdigest()}, "Q2 proposal snapshot digest; no compiled product claim")
    fields(receipt["platform"], "machine os_build release")
    actual_platform = q2_current_platform() if current_platform is None else current_platform
    require(receipt["platform"] == actual_platform and actual_platform["machine"] in ("arm64", "aarch64"), "Q2 platform changed")
    compiler, linker, sdk = receipt["compiler"], receipt["linker"], receipt["sdk"]
    fields(compiler, Q2_ID_FIELDS + " declared_dylibs flags reported_target target version")
    fields(linker, Q2_ID_FIELDS); fields(sdk, "path settings version")
    bounded_string(compiler["declared_dylibs"], 65536); bounded_string(compiler["version"], 65536)
    bounded_string(sdk["path"], 4096)
    require(compiler["mode"] & 0o100 and linker["mode"] & 0o100, "Q2 compiler/linker executable identity")
    require(compiler["target"] == "aarch64-apple-darwin" and re.fullmatch(r"(?:arm64|aarch64)-apple-darwin" + re.escape(actual_platform["release"]), compiler["reported_target"]), "Q2 compiler target/platform binding")
    require(re.fullmatch(r"[0-9]+(?:\.[0-9]+){1,2}", sdk["version"]), "Q2 SDK version")
    toolchain = Path(compiler["path"]).parent.parent
    sdk_root = Path(sdk["path"])
    require(toolchain.is_absolute() and sdk_root.is_absolute() and toolchain != Path("/") and sdk_root != Path("/")
            and str(sdk_root) == posixpath.normpath(sdk["path"]), "Q2 fixed compiler/SDK roots")
    roots = (toolchain, sdk_root)
    for pin in (compiler, linker, sdk["settings"]):
        identity = {k: pin[k] for k in Q2_ID_FIELDS.split()}
        q2_platform_file(identity, roots)
        platform_observed.append(identity)
    require(sdk["settings"]["path"] == str(sdk_root / "SDKSettings.json"), "Q2 SDK settings path binding")
    build, deps = receipt["build"], receipt["dependency_discovery"]
    fields(build, "argv cwd exit_code input output retained_stderr retained_stdout stderr stderr_sha256 stdout stdout_sha256")
    fields(deps, "argv exit_code retained_stdout stdout stdout_sha256")
    require(build["cwd"] == root["path"], "Q2 actual compiler retained cwd")
    for result in (build, deps):
        require(type(result["exit_code"]) is int and result["exit_code"] == 0, "Q2 actual preparation command failed")
        for stream in ("stdout", "stderr"):
            if stream in result:
                require(type(result[stream]) is str, "Q2 compiler log type")
                require(sha(result[stream].encode()) == result[stream + "_sha256"], "Q2 actual compiler log hash")
                retained = result["retained_" + stream]
                q2_identity_shape(retained)
                observed, wire = q2_private_file(root, retained["path"], expected=retained, collect=True)
                require(wire == result[stream].encode(), "Q2 actual retained compiler log bytes")
                private_observed.append(observed)
    flags = compiler["flags"]
    checked_array(flags)
    for flag in flags: bounded_string(flag, 4096)
    require(flags.count("-target") == flags.count("-isysroot") == 1
            and flags[flags.index("-target") + 1:flags.index("-target") + 2] == ["arm64-apple-darwin"]
            and flags[flags.index("-isysroot") + 1:flags.index("-isysroot") + 2] == [sdk["path"]], "Q2 selected compiler profile/SDK")
    argv = build["argv"]
    require(type(argv) is list and len(argv) == len(flags) + 4 and argv[:len(flags) + 1] == [compiler["path"], *flags] and argv[-2] == "-o", "Q2 exact recorded compile argv")
    compiled_source = Path(argv[len(flags) + 1])
    built_source, _ = q2_private_file(root, compiled_source, expected=build["input"])
    private_observed.append(built_source)
    require((built_source["len"], built_source["sha256"]) == (source["len"], source["sha256"]), "Q2 actual compiler source bytes")
    # Freezing may preserve a separately named immutable executable. The
    # original recorded output must still bind its actual bytes to that file;
    # an argv string or a same-version label alone supplies no such relation.
    compiled_output, _ = q2_private_file(root, argv[-1], expected=build["output"], executable=True)
    require((compiled_output["len"], compiled_output["sha256"]) == (exe["len"], exe["sha256"]), "Q2 actual compiler output differs from supplied executable")
    # The supplied dependency command has its own selected projection. It need
    # not repeat verbose-only flags or an invented reconstruction recipe.
    dep_argv = deps["argv"]
    require(type(dep_argv) is list and 0 < len(dep_argv) <= 64 and dep_argv[0] == compiler["path"]
            and dep_argv.count(str(compiled_source)) == 1 and dep_argv.count("-M") == 1
            and dep_argv.count("-MT") == 1 and dep_argv[dep_argv.index("-MT") + 1:dep_argv.index("-MT") + 2] == ["probe"], "Q2 exact selected header-discovery argv")
    text = deps["stdout"].replace("\\\n", " ")
    require(text.startswith("probe:") and "\\" not in text and all(c not in text for c in ("\r", "\x00")), "Q2 unsupported dependency path syntax")
    dependency_paths = text[len("probe:"):].split()
    require(len(dependency_paths) == len(set(dependency_paths)) and len(dependency_paths) <= 4096, "Q2 dependency closure bound/duplicates")
    headers = receipt["header_inputs"]
    require(type(headers) is dict and 0 < len(headers) <= 4096, "Q2 header closure bound")
    expected_dependencies = set(headers) | {str(compiled_source)}
    require(set(dependency_paths) == expected_dependencies, "Q2 incomplete dependency/header closure or unbound alias")
    require(sdk["settings"]["path"] in headers, "Q2 SDK settings absent from lexical dependency closure")
    for name, pin in headers.items():
        require(name == pin["path"], "Q2 header path binding")
        private_observed.extend(q2_header_file(root, pin, roots))
    closure = selection["build_closure"]
    require(type(closure) is list and 0 < len(closure) <= 64, "Q2 selected build closure missing")
    for pin in closure: q2_identity_shape(pin)
    require(len({p["path"] for p in closure}) == len(closure), "Q2 duplicate selected build input")
    # Recognize the one recorded linker invocation. Exact token comparison
    # handles both quoted executable names and unquoted archive/LTO inputs.
    linker_commands = []
    for line in build["stderr"].splitlines():
        if line.lstrip().startswith('"' + linker["path"] + '" '):
            try: tokens = shlex.split(line, posix=True)
            except ValueError as error: raise Refused("Q2 malformed recorded linker invocation") from error
            require(len(tokens) <= 4096, "Q2 recorded linker token bound")
            linker_commands.append(tokens)
    require(len(linker_commands) == 1, "Q2 missing/ambiguous recorded linker invocation")
    tokens = linker_commands[0]
    required_link_files = {token for token in tokens if token.startswith("/") and token.endswith((".a", ".dylib"))
                           and not token.startswith(("/usr/lib/", "/System/Library/"))}
    require({pin["path"] for pin in closure} == required_link_files, "Q2 incomplete selected archive/LTO build closure")
    for pin in closure:
        q2_platform_file(pin, roots)
        platform_observed.append(pin)
    declared = compiler["declared_dylibs"].splitlines()
    require(declared and declared[0] == compiler["path"] + ":", "Q2 compiler declared loads identity")
    for line in declared[1:]:
        match = re.fullmatch(r"\t(/[^\r\n]+) \(compatibility version [^\r\n]+\)", line)
        require(match and match[1].startswith(("/usr/lib/", "/System/Library/")), "Q2 unexpected nonplatform compiler load")
    actual_macho, install, _ = parse_macho(exe_bytes, binary=True)
    require(install is None and canonical(actual_macho) == canonical(selection["macho"]), "Q2 exact supplied executable Mach-O facts mismatch")
    retained_required = {pin["path"] for pin in private_observed} | {source["path"], exe["path"], compiled_output["path"]}
    q2_retained_outputs(root, receipt["retained_outputs"], retained_required)
    # Raw load facts and enumerated stable linker inputs do not prove resolved
    # transitive native closure, cache membership or compiler fidelity. Those
    # remain explicit independent Root-reviewed premises, not ingestion claims.
    # Recheck selected private file identities after all offline reads. A later
    # matching endpoint cannot undo a mismatch that already raised refusal.
    for name, executable in (("source", False), ("executable", True), ("preparation", False)):
        q2_private_file(root, selection[name]["path"], expected=selection[name], executable=executable,
                        limit=Q2_PREPARATION_LIMIT if name == "preparation" else MAX_ARCHIVE)
    for pin in private_observed: q2_private_file(root, pin["path"], expected=pin)
    q2_private_file(root, compiled_output["path"], expected=compiled_output, executable=True)
    for pin in platform_observed: q2_platform_file(pin, roots)
    for pin in headers.values(): q2_header_file(root, pin, roots)
    q2_retained_outputs(root, receipt["retained_outputs"], retained_required)
    require(receipt["platform"] == (q2_current_platform() if current_platform is None else current_platform), "Q2 observed platform change during ingestion")
    return {"source": source, "source_bytes": source_bytes, "executable": exe, "preparation": receipt_pin,
            "proposal_source_sha256": digest.hexdigest(), "scope": "offline input ingestion only; no launch or runtime grant"}


def selected_q2_input():
    return ingest_q2_input(ROOT_Q2_INPUT)



def source_lock_for_q2(q2_tuple):
    # Pure source-pin construction; an explicit tuple is also used by the
    # historical fixture unit test. Production source_lock() requires selection.
    fields(q2_tuple, "source sha256")
    validate_ref(q2_tuple["source"])
    require(q2_tuple["source"]["sha256"] == q2_tuple["sha256"], "Q2 source-lock cross-binding")
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
            "canonical_q2": q2_tuple}


def source_lock():
    selected = selected_q2_input()["source"]
    ref = {"path": "provenance/blobs/" + selected["sha256"], "len": selected["len"], "sha256": selected["sha256"]}
    return source_lock_for_q2({"source": ref, "sha256": selected["sha256"]})


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


def validate_lock(data, *, test_q2_tuple=None):
    lock = strict_json(data, 32768)
    # Equality to independently frozen pins includes every known field, order,
    # release root, signed byte form, key, patch preimage and canonical probe.
    expected = source_lock() if test_q2_tuple is None else source_lock_for_q2(test_q2_tuple)
    require(lock == expected, "source/key/patch/probe lock mismatch")
    return lock


def validate_proposal(data):
    selected = selected_q2_input()
    receipt = strict_json(data, 32768)
    fields(receipt, "schema source_sha256 os git gpg capability")
    require(receipt["schema"] == "bboxy.dream.proof-tools/1" and receipt["source_sha256"] == selected["proposal_source_sha256"], "proposal source/schema mismatch")
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
    require(capability["probe_source_sha256"] == selected["source"]["sha256"]
            and capability["probe_exe_sha256"] == selected["executable"]["sha256"], "canonical selected source/executable mismatch")
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
    require(type(q["logs"]) is list and len(q["logs"]) >= 3 and q["logs"][0]["path"] == "provenance/pretest-basis.json", "qualification must retain actual Basis/descriptor/inventory")
    basis_data = ref_bytes(prefix, q["logs"][0], 49152)
    descriptor_data = ref_bytes(prefix, q["logs"][1], 16384)
    prepared_budget(data, lock_data, report, basis_data, descriptor_data)
    basis = strict_json(basis_data, 49152)
    basis_platform_current(basis)
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
    selected = selected_q2_input()
    require(q["build_id"] == intent["build_id"] == prefix.name and q["incarnation"] == intent["incarnation"], "candidate birth/incarnation mismatch")
    require(q["builder"]["source_sha256"] == inputs["source_sha256"], "builder source/recipe currentness mismatch")
    require(q["builder"]["commit"] == inputs["commit"], "builder source commit changed")
    require(q["platform"]["os_build"] == inputs["os_build"] and q["platform"]["release"] == inputs["release"] and q["platform"]["arch"] == inputs["arch"], "OS/platform currentness mismatch")
    basis = strict_json(ref_bytes(prefix, q["logs"][0], 49152), 49152)
    basis_platform_current(basis, inputs["shared_cache_uuid"])
    require(receipt["os"] == {"release": inputs["release"], "arch": inputs["arch"]}, "proposal OS binding mismatch")
    require(q["compiler"]["path"] == inputs["compiler"] and q["compiler"]["sha256"] == inputs["compiler_sha256"], "compiler currentness mismatch")
    require(q["sdk"] == {"path": inputs["sdk"], "version": inputs["sdk_version"], "settings_sha256": inputs["sdk_settings_sha256"]}, "SDK currentness mismatch")
    require(q["patches"] == source_lock()["profile_patches"], "profile patch binding mismatch")
    require(q["component"]["authority_source"]["sha256"] == receipt["capability"]["probe_source_sha256"] == selected["source"]["sha256"]
            and q["component"]["authority_source"]["len"] == selected["source"]["len"], "canonical component source binding mismatch")
    require(q["component"]["probe_exe"]["sha256"] == receipt["capability"]["probe_exe_sha256"] == selected["executable"]["sha256"], "canonical component executable binding mismatch")
    for name in ("git", "gpg"):
        require(q["tools"][name]["path"] == "bin/" + name and receipt[name]["absolute_path"] == str(prefix / "bin" / name) and q["tools"][name]["sha256"] == receipt[name]["sha256"], "tool image/path binding mismatch")


def check_inventory(prefix, q):
    check_private_dir(prefix)
    allowed = {"qualification.json", "receipt.candidate.json"} | {ref["path"] for ref in file_refs(q)}
    inventory = strict_json(ref_bytes(prefix, q["logs"][-1]), MAX_TAR)
    require(type(inventory) is list and len(inventory) <= 40000, "offline inventory log bound")
    seen = []
    for ref in inventory:
        validate_ref(ref)
        ref_bytes(prefix, ref)
        seen.append(ref["path"])
        allowed.add(ref["path"])
    require(seen == sorted(set(seen)), "offline inventory log order/duplicates")
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
    # Consume the original sealed witness; a test command never repeats a
    # timing attempt, recompiles Q2, or replaces an unresolved original effect.
    selected_q2_input()
    run = session(d, "component-check")
    private_dir(run / "logs"); private_dir(run / "public-home")
    inputs = build_inputs(run)
    previous = previous_build(d, inputs, run.name)
    require(previous is not None, "no sealed current-source candidate; run the authorized build first")
    intent = strict_json(read_regular(previous / "intent.json", 32768, True), 32768)
    locator, q = validate_candidate(Path(intent["prefix"]), intent, inputs)
    require(q["candidate_status"] == "review_ready", "original component evidence is unsupported")
    exclusive(run / "component-check.json", canonical(locator))
    return locator


def private_descriptor(run, prefix, basis, case, output, selection):
    """Produce the existing ICD input; only Q2 decodes/runs its commands."""
    require(case in ("positive", "corrupt_proof", "pause_resume", "cancel", "stale_incarnation",
                     "clock_failure", "debt_overlap", "reap_delay"), "unknown canonical component case")
    shared = file_pin(prefix, "provenance/pretest-basis.json")
    fixture = basis["fixture"]
    value = {"kind": "private_candidate_test", "run_id": ulid(), "nonce": ulid(),
             "incarnation": basis["build"]["incarnation"], "deadline_ns": time.clock_gettime_ns(time.CLOCK_MONOTONIC) + 30_000_000_000,
             "os_build": basis["build"]["platform"]["os_build"], "arch": "aarch64",
             "probe_source_sha256": selection["source"]["sha256"], "probe_exe_sha256": selection["executable"]["sha256"],
             "prefix": dir_pin(prefix), "source_lock": file_pin(prefix, basis["build"]["source_lock"]["path"]),
             **{name: dict(shared) for name in ("build", "source_review", "closure", "config", "commands")},
             "git": file_pin(prefix, "bin/git"), "gpg": file_pin(prefix, "bin/gpg"),
             "fixture": {"repo": fixture["repo"], "manifest": dict(shared),
                         "valid_commit": fixture["valid_commit"], "corrupt_commit": fixture["corrupt_commit"],
                         "public_keys": file_pin(Path(fixture["repo"]["path"]), fixture["public_keys"]["path"], "fixture")},
             "output": dir_pin(output), "case": case}
    raw = canonical(value)
    # Reserve the raw result before launch. The final envelope is checked
    # against the SAME aggregate again before any publication.
    prepared_budget(b"", ref_bytes(prefix, basis["build"]["source_lock"]), b"\0" * 65536,
                    read_regular(prefix / shared["path"], 49152, True), raw)
    return value, raw


def wait_component(child, case_dir, deadline_ns):
    """Hold the original child until actual wait4, including after refusal wire.

    This is the offline test parent's wait, not another helper controller. It
    issues no signals, timeout kill, replacement launch or settlement claim.
    A live Q2 quarantine keeps BOTH owners alive and the exact Popen handle held.
    """
    announced = False
    while True:
        try:
            pid, status, usage = os.wait4(child.pid, os.WNOHANG)
        except OSError as error:
            # Losing actual wait ownership is unresolved, never proof of exit.
            # Do not retry an unknown reaping effect or let this parent die.
            print(f"OWNED UNKNOWN: component pid={child.pid} wait4={error}; handle retained at {case_dir}", flush=True)
            while True:
                time.sleep(1)
        if pid:
            require(pid == child.pid, "wait4 returned a foreign component identity")
            child.returncode = os.waitstatus_to_exitcode(status)
            record = {"pid": pid, "wait_status": status, "exit_code": child.returncode,
                      "user_cpu_ns": int(usage.ru_utime * 1_000_000_000),
                      "system_cpu_ns": int(usage.ru_stime * 1_000_000_000),
                      "max_rss_bytes": usage.ru_maxrss, "actual_wait4": True}
            exclusive(case_dir / "wait4.json", canonical(record))
            return record
        if not announced and time.clock_gettime_ns(time.CLOCK_MONOTONIC) >= deadline_ns:
            # A result file is deliberately NOT an exit indication. Keep the
            # process/parent alive until the canonical owner proves settlement.
            try:
                exclusive(case_dir / "owned-pending.json", canonical({"pid": child.pid,
                          "controller_reaped": False, "effect": "original_controller_handle_retained"}))
            except OSError:
                # Even a failed pending receipt cannot abandon actual custody.
                pass
            print(f"OWNED PENDING: component controller pid={child.pid}; retained {case_dir}; no timeout kill or retry", flush=True)
            announced = True
        time.sleep(0.01)


def component_result(raw, descriptor, actual_exit):
    """Check the selected producer's verdict/binding, without another decoder.

    Q2 owns the complete result schema and causal engine. These checks cannot
    mint a result, rewrite a refusal or turn private evidence into authority.
    """
    require(0 < len(raw) <= 65536 and raw.endswith(b"\n"), "canonical Q2 result/LF byte bound")
    result = strict_json(raw[:-1], 65535)
    require(result["kind"] == "private_candidate_result" and result["profile"] == "read-helper-native/1"
            and result["runtime_approved"] is False and result["availability"] == "not_applicable_private_test",
            "component output is not unapproved private evidence")
    for key in ("run_id", "nonce", "incarnation", "case"):
        require(result[key] == descriptor[key], "component original operation binding: " + key)
    require(result["input_sha256"] == sha(canonical(descriptor)), "component input hash binding")
    require(result["probe"] == {"source_sha256": descriptor["probe_source_sha256"], "exe_sha256": descriptor["probe_exe_sha256"]}, "component probe binding")
    require(result["binding"] == {key: descriptor[key]["sha256"] for key in
            ("source_lock", "build", "source_review", "closure", "config", "commands", "git", "gpg")}, "component pretest binding")
    require(result["basis"] == {"kind": "conditional_on_bound_source_review", "review_sha256": descriptor["source_review"]["sha256"],
            "whole_command_lifetime_proof": False}, "component conditional source premise")
    case = descriptor["case"]
    if case in ("positive", "corrupt_proof", "pause_resume"):
        corrupt = case == "corrupt_proof"
        require(actual_exit == (81 if case == "pause_resume" else 0) and result["phase"] == "settled"
                and result["outcome"] == ("expected_rejection" if corrupt else "completed")
                and result["observed_proof"] == ("invalid" if corrupt else "valid") and result["error"] is None,
                "canonical real proof/component verdict failed; no timing retry")
        require(result["settlement"] == {"created": 16, "reaped": 16, "group_gone": True, "retired_coverage": True}, "actual complete component settlement")
        metrics = result["metrics"]
        for key, bound in (("ordinary_max_ns", 1000000), ("controller_max_ns", 1000000), ("cleanup_max_ns", 1000000),
                           ("native_cpu_max_ns", 2000000), ("native_wall_max_ns", 250000000), ("inclusive_max_ns", 3000000),
                           ("A_ns", 3000000000), ("helper_group_rss_peak_bytes", 16 * 1024 * 1024), ("probe_rss_peak_bytes", 96 * 1024 * 1024)):
            checked_int(metrics[key], 0, bound)
        # Decision66331: this producer's native_wall_max_ns is populated only
        # by engine_birth/prepared_spawn (process_create), not other classes.
        # Decision66280: A is authoritative rounded leader-wait4 CPU including
        # qualified child roll-up. The exact bound producer checks each command
        # <=250ms, job <=3s, live QoS/CPU/RSS and post-reap helper debt. Retired
        # per-grant Q/W fields can be null and provide no qualification evidence.
        checked_int(metrics["C_ns"], 0, 2**64 - 2)
        require(metrics["attempts"] == 16 and metrics["members_max"] == 2
                and type(metrics["debt_charged_ns"]) is int and metrics["debt_charged_ns"] == metrics["debt_paid_ns"], "component debt/attempt/member conjunction")
    else:
        expected = {"cancel": "cancelled", "stale_incarnation": "coverage_unknown", "clock_failure": "clock_unknown",
                    "debt_overlap": "coverage_unknown", "reap_delay": "settlement_unknown"}
        require(actual_exit == 81 and result["error"] is not None and result["error"]["code"] == expected[case]
                and result["settlement"]["created"] > 0 and result["metrics"]["attempts"] > 0,
                "canonical causal control did not reach its named engine failure")
    return result


def component_terminal(stderr, case, result=None, actual_exit=None):
    # Consume private_finish_diagnostics from the exact selected producer. Its
    # final receipt follows the paid tail and complete error-code diagnostics;
    # its own write remains the explicit cutoff, corroborated by parent wait4.
    require(stderr.endswith(b"\n"), "canonical terminal stderr is incomplete")
    lines = stderr.decode("utf-8", errors="strict").splitlines()
    terminal = [i for i, line in enumerate(lines) if line.startswith("private_candidate_terminal ")]
    tail = [i for i, line in enumerate(lines) if line.startswith("private_candidate_exit_tail ")]
    require(len(terminal) == len(tail) == 1 and tail[0] < terminal[0] == len(lines) - 1
            and not any(line.startswith("private_candidate_terminal_gate=failed") for line in lines),
            "canonical final receipt must follow the paid tail exactly once")
    def values(line):
        words = line.split(" ")[1:]
        require(all(word and word.count("=") == 1 for word in words), "canonical terminal receipt tokens")
        pairs = [word.split("=", 1) for word in words]
        require(len({p[0] for p in pairs}) == len(pairs), "canonical terminal duplicate token")
        return dict(pairs)
    def number(value):
        require(type(value) is str and re.fullmatch(r"0|[1-9][0-9]{0,19}", value) is not None, "canonical terminal integer")
        parsed = int(value); checked_int(parsed, 0, 2**64 - 1)
        return parsed
    row, end = values(lines[terminal[0]]), values(lines[tail[0]])
    fields(end, "output_closed close_max_ns debt_charged_ns debt_paid_ns ordinary_max_ns")
    fields(row, "profile stage metrics_cutoff metadata_kind terminal_cutoff runtime_approved helper_created helper_reaped result_complete control_verdict persistence_verdict terminal_cost_verdict tail_complete persistence_attempted persistence_open persistence_write persistence_fsync persistence_close persistence_directory_fsync persistence_errno stdout_errno stdout_attempted reporting_only original_entry_ns original_deadline_ns deadline_observed_ns deadline_first_ns deadline_first_unit_ns deadline_first_line deadline_last_line reporting_started_ns closed_unit_count path_walk_components path_walk_units booked_ordinary_units engine_charge_calls poll_closure_calls ordinary_cost_ns ordinary_raw_cpu_ns ordinary_raw_complete ordinary_floor_surcharge_ns helper_cost_total_ns debt_components_match debt_scope process_create_wall_excess_ns process_create_wall_excess_count process_create_wall_excess_known process_create_wall_excess_scope persistence_paid formatter_cpu_start_ns formatter_cpu_end_ns formatter_inclusive_ns write_cpu_start_ns write_cpu_end_ns write_inclusive_ns close_max_cpu_ns probe_rss_peak_bytes rss_known ordinary_max_ns debt_charged_ns debt_paid_ns retained_units retained_overflow format_paid write_paid error_count error_codes error_counts errors_complete diagnostics_complete parent_actual_reap_required")
    # Frozen result-path C:424-426/452/3934-3940: one textual boolean,
    # one terminal reporting flag, two unsigned32 source lines and nine uint64
    # observations. C:610-616 records first-unit CPU only after its 1ms gate.
    # Zero clock/line sentinels are retained; these fields waive no verdict.
    require(row["stdout_attempted"] in ("true", "false"), "canonical terminal stdout-attempt boolean")
    require(number(row["reporting_only"]) == 1, "canonical terminal reporting flag")
    for name in ("deadline_first_line", "deadline_last_line"):
        require(number(row[name]) <= 2**32 - 1, "canonical terminal source-line range")
    for name in ("original_entry_ns", "original_deadline_ns", "deadline_observed_ns", "deadline_first_ns",
                 "reporting_started_ns", "closed_unit_count", "path_walk_components", "path_walk_units"):
        number(row[name])
    require(number(row["deadline_first_unit_ns"]) <= 1000000, "canonical terminal first expired unit CPU range")
    # INPUT54 C:3970-3978, authorized by the task's exact ten-field ruling.
    for name in ("booked_ordinary_units", "engine_charge_calls", "poll_closure_calls", "ordinary_cost_ns",
                 "ordinary_raw_cpu_ns", "ordinary_floor_surcharge_ns", "helper_cost_total_ns"):
        number(row[name])
    for name in ("ordinary_raw_complete", "debt_components_match"):
        require(row[name] in ("true", "false"), "canonical terminal INPUT54 boolean")
    require(row["debt_scope"] == "whole_controller_cumulative", "canonical terminal INPUT54 debt scope")
    # INPUT60's exact four-field observation; unknown remains numeric0.
    number(row["process_create_wall_excess_ns"])
    number(row["process_create_wall_excess_count"])
    require(number(row["process_create_wall_excess_known"]) <= 1, "canonical terminal process-create excess flag")
    require(row["process_create_wall_excess_scope"] == "invocation_sum_once_per_prepared_spawn",
            "canonical terminal process-create excess scope")
    require(row["profile"] == "read-helper-native/1" and row["metrics_cutoff"] == "pre_result_emission"
            and row["metadata_kind"] == "declared_peak_capacity"
            and row["terminal_cutoff"] == "after_paid_tail_and_error_diagnostics_before_final_receipt"
            and row["runtime_approved"] == "false", "canonical independent terminal verdict source/cutoff")
    for name in ("result_complete", "tail_complete", "persistence_attempted", "persistence_open", "persistence_write",
                 "persistence_fsync", "persistence_close", "persistence_directory_fsync", "persistence_paid", "rss_known",
                 "format_paid", "write_paid", "errors_complete", "diagnostics_complete", "parent_actual_reap_required"):
        require(row[name] == "true", "canonical terminal incomplete observation: " + name)
    require(row["persistence_verdict"] == "persisted" and row["terminal_cost_verdict"] == "passed"
            and number(row["persistence_errno"]) == number(row["stdout_errno"]) == 0
            and end["output_closed"] == "true", "canonical independent persistence or terminal-cost refusal")
    for stem in ("formatter", "write"):
        before, after = number(row[stem + "_cpu_start_ns"]), number(row[stem + "_cpu_end_ns"])
        require(before <= after and after - before <= 1000000 and number(row[stem + "_inclusive_ns"]) <= 1000000,
                "canonical terminal ordinary span exceeds gate")
    require(number(row["formatter_cpu_end_ns"]) <= number(row["write_cpu_start_ns"])
            and number(row["close_max_cpu_ns"]) <= 1000000 and number(end["close_max_ns"]) <= 1000000,
            "canonical terminal order/close gate")
    debt = number(row["debt_charged_ns"])
    require(debt == number(row["debt_paid_ns"]) and number(end["debt_charged_ns"]) == number(end["debt_paid_ns"]) <= debt
            and number(row["retained_units"]) == number(row["retained_overflow"]) == 0,
            "canonical terminal unpaid or retained debt")
    require(number(row["probe_rss_peak_bytes"]) <= 96 * 1024 * 1024, "canonical terminal RSS limit")
    created, reaped = number(row["helper_created"]), number(row["helper_reaped"])
    require(0 < created == reaped <= 256, "canonical terminal helpers lack actual settlement")
    success_case = case in ("positive", "corrupt_proof", "pause_resume")
    expected = {"cancel": "cancelled", "stale_incarnation": "coverage_unknown", "clock_failure": "clock_unknown",
                "debt_overlap": "coverage_unknown", "reap_delay": "settlement_unknown"}
    require(success_case or case in expected, "unknown canonical terminal case")
    require(row["control_verdict"] == ("passed" if success_case else "failed"), "canonical independent control verdict")
    codes = ("cancelled", "clock_unknown", "coverage_unknown", "deadline", "input_invalid", "resource_limit",
             "review_unbound", "settlement_unknown", "signature_unexpected", "source_changed", "other")
    require(row["error_codes"] == ",".join(codes), "canonical terminal error-code ordering")
    counts = row["error_counts"].split(",")
    require(len(counts) == len(codes), "canonical terminal error-count shape")
    histogram = {code: number(value) for code, value in zip(codes, counts)}
    require(sum(histogram.values()) == number(row["error_count"]), "canonical complete error-count sum")
    errors = {}
    for index, line in enumerate(lines):
        if not line.startswith('{"code":'): continue
        observation = strict_json(line.encode(), 2304)
        fields(observation, "code count details_truncated first_detail kind last_detail")
        require(observation["kind"] == "private_candidate_error" and observation["code"] in codes
                and tail[0] < index < terminal[0] and observation["code"] not in errors,
                "canonical error diagnostic identity/order")
        checked_int(observation["count"], 1, 2**64 - 1)
        require(observation["details_truncated"] is False, "canonical error detail coverage is truncated")
        for name in ("first_detail", "last_detail"):
            require(type(observation[name]) is str and len(observation[name].encode()) <= 160, "canonical error detail bound")
        errors[observation["code"]] = observation
    require(list(errors) == [code for code in codes if histogram[code]]
            and all(observation["count"] == histogram[code] for code, observation in errors.items()),
            "canonical complete error diagnostics disagree with final histogram")
    require(sum(len((line + "\n").encode()) for line in lines[tail[0]:]) <= 32768, "canonical terminal diagnostic byte bound")
    if success_case:
        require(not errors and created == 16 and row["stage"] == "settled"
                and number(row["ordinary_max_ns"]) <= 1000000 and number(end["ordinary_max_ns"]) <= 1000000,
                "canonical successful control carries additional errors or cost")
    else:
        require(histogram[expected[case]] > 0, "canonical causal control error is absent")
        allowed = {expected[case]}
        # These auxiliary failures are explicit consequences of the selected
        # producer's injected unknown-clock/quarantine path, before terminal
        # observation starts. Other error classes remain an additional failure.
        if case == "clock_failure":
            allowed.add("resource_limit")
            if "resource_limit" in errors:
                # Decision66331 changed this process-create wall gate to250ms.
                # Keep the current frozen producer's exact auxiliary detail.
                require(errors["resource_limit"]["first_detail"] == errors["resource_limit"]["last_detail"]
                        == "process creation native2ms wall250ms application1ms inclusive3ms gate", "additional resource failure in clock control")
        if case == "reap_delay":
            allowed.add("clock_unknown")
            if "clock_unknown" in errors:
                known = {"unknown or overflowing closed-unit cost", "closed engine unit has unknown CPU"}
                require(errors["clock_unknown"]["first_detail"] in known and errors["clock_unknown"]["last_detail"] in known,
                        "additional clock failure in reap control")
        require(set(errors) <= allowed, "canonical control has additional error classes")
        # These may retain the deliberately failed engine metric. Their final
        # independent terminal-cost verdict above must still pass.
        number(row["ordinary_max_ns"]); number(end["ordinary_max_ns"])
    if result is not None:
        # Decision66074 retains the same original handle after raw refusal.
        # The injected first-helper reap delay emits quarantine before hold
        # settlement; its later terminal may record settled1/1 after wait4.
        late_reap = (case == "reap_delay" and actual_exit == 81 and row["stage"] == "settled"
                     and created == reaped == 1 and result["phase"] == result["outcome"] == "quarantined"
                     and result["observed_proof"] == "not_reached"
                     and type(result["metrics"]["attempts"]) is int and result["metrics"]["attempts"] == 1
                     and canonical(result["settlement"]) == canonical(
                         {"created": 1, "reaped": 0, "group_gone": False, "retired_coverage": False}))
        require((row["stage"] == result["phase"] or late_reap) and created == result["settlement"]["created"]
                and reaped >= result["settlement"]["reaped"], "canonical final settlement/result binding")
        require((result["error"] is None) == success_case, "canonical final control/result error binding")
        if not success_case: require(result["error"]["code"] == expected[case], "canonical first control error differs")
    if actual_exit is not None:
        require(actual_exit == (0 if case in ("positive", "corrupt_proof") else 81), "canonical final verdict/actual exit binding")
    return {"terminal": row, "exit_tail": end, "errors": errors}


def run_private_components(run, prefix, basis):
    selected = selected_q2_input()
    cases = private_dir(run / "components")
    records = []
    for name in ("positive", "corrupt_proof", "pause_resume", "cancel", "stale_incarnation", "clock_failure", "debt_overlap", "reap_delay"):
        case = private_dir(cases / name)
        output = private_dir(case / "output")
        descriptor, raw = private_descriptor(run, prefix, basis, name, output, selected)
        path = case / "pretest-input.json"; exclusive(path, raw)
        argv = [selected["executable"]["path"], "private-candidate", str(path), sha(raw)]
        exclusive(case / "spawn-intent.json", canonical({"argv": argv, "input_sha256": sha(raw), "build_id": run.name}))
        # Files avoid a pipe deadlock while Q2 retains a live quarantine. Only
        # the immutable selected executable is launched; it is never rebuilt.
        with open(case / "stdout", "xb") as stdout, open(case / "stderr", "xb") as stderr:
            child = subprocess.Popen(argv, cwd=case, env=env(case), stdin=subprocess.DEVNULL,
                                     stdout=stdout, stderr=stderr, start_new_session=True)
            record_error = None
            try:
                exclusive(case / "pid.json", canonical({"pid": child.pid, "pgid": child.pid, "input_sha256": sha(raw)}))
            except OSError as error:
                # A failed custody write cannot drop the already born child.
                record_error = error
            waited = wait_component(child, case, descriptor["deadline_ns"])
            if record_error is not None:
                raise record_error
            stdout.flush(); os.fsync(stdout.fileno()); stderr.flush(); os.fsync(stderr.fileno())
        report = read_regular(case / "stdout", 65536, True)
        # Retain original raw bytes and all physical results even if semantic
        # checks below refuse. No failed original operation is redispatched.
        record = {"case": name, "descriptor": retained_file(prefix, path), "report": retain_blob(prefix, report),
                  "stderr": retained_file(prefix, case / "stderr"), "wait4": retained_file(prefix, case / "wait4.json"),
                  "actual_exit": waited["exit_code"], "argv": argv}
        ledger = output / "ledger.ndjson"
        if ledger.exists(): record["ledger"] = retained_file(prefix, ledger)
        exclusive(case / "retained.json", canonical(record))
        result = component_result(report, descriptor, waited["exit_code"])
        component_terminal(read_regular(case / "stderr", MAX_TAR, True), name, result, waited["exit_code"])
        require(read_regular(output / "result.json", 65536, True) == report, "persisted Q2 result differs from unaltered stdout")
        records.append(record)
        exclusive(case / "checked.json", canonical({"check_exit": 0, "actual_exit": waited["exit_code"], "outcome": result["outcome"], "runtime_approved": False}))
        print(f"canonical private component {name} actual_exit={waited['exit_code']} check_exit=0", flush=True)
    exclusive(run / "component-checks.json", canonical(records))
    return records


def qualify_candidate(run, prefix, basis):
    records = run_private_components(run, prefix, basis)
    # The frozen task orders components, final canonical gates, qualification,
    # then receipt. A gate failure retains this same original build effect.
    command(run, ["/usr/bin/make", "test-formal-parity", "test-licenses",
                  "TARGET_DIR=" + str(run.parents[2])], cwd=ROOT,
            extra_env={"PATH": "/Users/feral/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin",
                       "CARGO_HOME": "/Users/feral/.cargo", "RUSTUP_HOME": "/Users/feral/.rustup",
                       "CARGO_BUILD_JOBS": "2"})
    first = records[0]
    selected = selected_q2_input()
    build = basis["build"]
    source = strict_json(read_regular(run / "intent.json", 32768, True), 32768)["inputs"]
    review = []
    for finding in basis["review"]["findings"]:
        refs = [retained_file(prefix, run / "src" / next(p[0] + "-" + p[1] for p in PINS if p[0] == span["source"]) / span["path"]) for span in finding["spans"]]
        review.append({"id": finding["id"], "source_refs": refs, "finding": "established",
                       "reason": "Finite coverage and exact patched-source binding established; conditional source/compiler/platform premise only. " + finding["rationale"]})
    basis_ref = {"path": "provenance/pretest-basis.json", "len": len(canonical(basis)), "sha256": sha(canonical(basis))}
    logs = [basis_ref, first["descriptor"]]
    for record in records:
        logs += [record[key] for key in ("report", "stderr", "wait4", "ledger") if key in record]
    q = {"format": "bboxy.dream.proof-tool-qualification/1", "build_id": run.name,
         "incarnation": build["incarnation"], "builder": build["builder"], "source_lock": build["source_lock"],
         "sources": [{key: value for key, value in row.items() if key != "status"} for row in build["auth"]],
         "patches": source_lock()["profile_patches"],
         "compiler": {"path": build["compiler"]["image"]["path"], "sha256": build["compiler"]["image"]["sha256"], "version": build["compiler"]["version"]},
         "sdk": {"path": build["sdk"]["path"], "version": build["sdk"]["version"], "settings_sha256": build["sdk"]["settings"]["sha256"]},
         "platform": {key: build["platform"][key] for key in ("os_build", "release", "arch")},
         "tools": {name: {"path": "bin/" + name, "len": build["tools"][name]["len"], "sha256": build["tools"][name]["sha256"]} for name in ("git", "gpg")},
         "loaded_closure": [{"file": image["file"], "kind": "private", "identity": image["macho"]["uuid"], "assumption": "conditional native/compiler/platform/source premise"} for image in basis["closure"]["images"]],
         "configs": [row["config"] for row in build["generated"]], "command_profile_sha256": sha(canonical(basis["commands"])),
         "review": review, "component": {"authority_source": retain_blob(prefix, selected["source_bytes"]),
                    "probe_exe": retain_blob(prefix, read_regular(selected["executable"]["path"])), "report": first["report"]},
         "checks": [{"command": "component_result(" + row["case"] + ", actual_exit=" + str(row["actual_exit"]) + ")",
                     "exit_code": 0, "source_sha256": sha(SCRIPT_BYTES), "log": row["report"]} for row in records],
         "logs": logs, "candidate_status": "review_ready"}
    q["platform"]["assumptions"] = [{"id": "conditional_source_compiler_platform", "evidence": retain_blob(prefix, canonical(basis["closure"]["platform"]))}]
    # Offline inventory log binds all retained provenance (including transitive
    # Basis/source-lock Refs). It is not another runtime authority/catalogue.
    inventory = []
    for path in sorted(prefix.rglob("*")):
        if not path.is_dir():
            data = read_regular(path, MAX_TAR, True)
            inventory.append({"path": path.relative_to(prefix).as_posix(), "len": len(data), "sha256": sha(data)})
    q["logs"].append(retain_blob(prefix, canonical(inventory)))
    qbytes = canonical(q)
    prepared_budget(qbytes, ref_bytes(prefix, q["source_lock"]), ref_bytes(prefix, first["report"]),
                    canonical(basis), ref_bytes(prefix, first["descriptor"]))
    proposal = {"schema": "bboxy.dream.proof-tools/1", "source_sha256": selected["proposal_source_sha256"],
                "os": {"release": source["release"], "arch": source["arch"]},
                **{name: {"absolute_path": str(prefix / "bin" / name), "sha256": q["tools"][name]["sha256"],
                          "build": next(p[1] for p in PINS if p[0] == ("gnupg" if name == "gpg" else name)),
                          "features": []} for name in ("git", "gpg")},
                "capability": {"probe_source_sha256": selected["source"]["sha256"], "probe_exe_sha256": selected["executable"]["sha256"],
                               "evidence_sha256": sha(qbytes), "features": []}}
    locator = publish_candidate(run, prefix, q, proposal, source)
    print(canonical(locator).decode(), flush=True)
    return locator


# Inert OFFLINE input-ingestion fixtures. No synthetic signature assertion is
# cryptographic evidence; production selection is never exercised by these tests.
SYNTHETIC_Q2_PLATFORM = {"machine": "arm64", "release": "27.0.0", "os_build": "26A434"}
SYNTHETIC_Q2_COMMIT = "1" * 40
SYNTHETIC_Q2_SIGNER = "9B07E849480C68328CA20684844ECC0587DE7AB0"  # label only, NOT a signature verdict


def _synthetic_q2_write(root, path, data, mode=0o600):
    """Create only a fixture-owned path; all intermediate directories are0700."""
    root, path = Path(root), Path(path)
    parent = root
    for part in path.parent.relative_to(root).parts:
        parent = parent / part
        if not parent.exists():
            parent.mkdir(mode=0o700)
        info = parent.lstat()
        if not stat.S_ISDIR(info.st_mode) or stat.S_IMODE(info.st_mode) != 0o700:
            raise AssertionError("synthetic fixture parent is not a private directory")
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_NOFOLLOW, mode)
    try:
        view = memoryview(data)
        while view:
            amount = os.write(fd, view)
            if amount <= 0:
                raise AssertionError("incomplete synthetic fixture write")
            view = view[amount:]
        os.fchmod(fd, mode)
    finally:
        os.close(fd)


def _synthetic_q2_overwrite(path, data, mode=None):
    """Mutate one owned unit-fixture file, never a production selection."""
    path = Path(path)
    original_mode = stat.S_IMODE(path.lstat().st_mode)
    os.chmod(path, 0o600)
    fd = os.open(path, os.O_WRONLY | os.O_TRUNC | os.O_NOFOLLOW)
    try:
        view = memoryview(data)
        while view:
            amount = os.write(fd, view)
            if amount <= 0:
                raise AssertionError("incomplete synthetic mutation")
            view = view[amount:]
        os.fchmod(fd, original_mode if mode is None else mode)
    finally:
        os.close(fd)


def _synthetic_q2_pin(root_pin, path, executable=False):
    # This exercises the actual common11-field snapshot, not a fixture mirror.
    identity, content = q2_private_file(
        root_pin, str(path), executable=executable, collect=True
    )
    if len(content) != identity["len"]:
        raise AssertionError("synthetic snapshot collection mismatch")
    return identity


def _synthetic_q2_receipt(selection):
    return json.loads(Path(selection["preparation"]["path"]).read_bytes())


def _synthetic_q2_reseal_receipt(selection, receipt):
    """Seal a locally authored MALFORMED unit fixture before checking it.

    This intentionally allows an internally inconsistent receipt to have a
    matching outer hash, so tests reach inner cross-binding checks. It never
    changes ROOT_Q2_INPUT or an actual Root-selected tuple/authority constant.
    """
    path = Path(selection["preparation"]["path"])
    raw = json.dumps(
        receipt, sort_keys=True, separators=(",", ":"),
        ensure_ascii=False, allow_nan=False,
    ).encode() + b"\n"
    _synthetic_q2_overwrite(path, raw)
    selection["preparation"] = _synthetic_q2_pin(selection["root"], path)
    selection["bindings"] = {
        name: q2_metadata_hash(receipt[name]) for name in Q2_BINDINGS.split()
    }
    return selection


def synthetic_q2_input(root):
    """Return one explicit, inert internal-test selection dictionary.

    The source and preparation snapshots match the corrected receipt. The two public shared
    authority files are copied privately; every other input/log/image is clearly
    synthetic. Source length intentionally differs from the old39130B source.
    Captured identity dictionaries come only from q2_private_file(...collect).
    """
    root = Path(root).resolve()
    if not root.exists():
        root.mkdir(mode=0o700)
    if stat.S_IMODE(root.stat().st_mode) != 0o700:
        raise AssertionError("synthetic root must be0700")
    if any(character in str(root) for character in " \t\r\n\\"):
        raise AssertionError("synthetic dependency fixture uses unescaped private paths")
    root_pin = dir_pin(root)
    snapshot = root / "signed-inputs-synthetic-not-signed"
    preparation_snapshot = root / "preparation-inputs-synthetic-not-signed"
    preparation_commit = "3" * 40
    inputs = {}
    for name in Q2_INPUT_NAMES:
        path = (preparation_snapshot if name in Q2_PREPARATION_NAMES else snapshot) / name
        if name.endswith("closure-boundary.schema.json"):
            data = read_regular(FIX / "closure-boundary.schema.json", MAX_ARCHIVE)
            if sha(data) != CLOSURE_SCHEMA_SHA:
                raise AssertionError("actual shared schema changed")
        elif name.endswith("closure-boundary-vectors.json"):
            data = read_regular(FIX / "closure-boundary-vectors.json", MAX_ARCHIVE)
            if sha(data) != CLOSURE_VECTORS_SHA:
                raise AssertionError("actual shared vectors changed")
        elif name == Q2_C_SOURCE:
            data = b"/* INERT Q2 ingestion unit fixture; never compiled. */\n" + b" " * 40_100
        else:
            data = ("/* INERT input-ingestion fixture: " + name + " */\n").encode()
        _synthetic_q2_write(root, path, data, 0o400)
        pin = _synthetic_q2_pin(root_pin, path)
        inputs[name] = {"len": pin["len"], "sha256": pin["sha256"]}

    source_path = snapshot / Q2_C_SOURCE
    source = _synthetic_q2_pin(root_pin, source_path)
    source_bytes = source_path.read_bytes()
    compiled_source = root / "proof-helper-probe.c"
    _synthetic_q2_write(root, compiled_source, source_bytes, 0o400)

    compiler_path = root / "toolchain" / "usr" / "bin" / "clang"
    linker_path = root / "toolchain" / "usr" / "bin" / "ld"
    runtime_path = root / "toolchain" / "usr" / "lib" / "runtime.a"
    sdk_root = root / "SDK" / "MacOSX27.0.sdk"
    settings_path = sdk_root / "SDKSettings.json"
    header_path = sdk_root / "usr" / "include" / "inert.h"
    resolved_header = header_path.parent / "real" / "inert.h"
    header_snapshot = root / "header-snapshots" / "inert.h"
    _synthetic_q2_write(root, compiler_path, b"INERT compiler identity; never executed\n", 0o700)
    _synthetic_q2_write(root, linker_path, b"INERT linker identity; never executed\n", 0o700)
    _synthetic_q2_write(root, runtime_path, b"INERT archive identity; not a real archive\n")
    _synthetic_q2_write(root, settings_path, b'{"SyntheticIngestionOnly":true}\n')
    header_bytes = b"/* INERT retained SDK header, not compiled */\n"
    _synthetic_q2_write(root, resolved_header, header_bytes)
    header_path.symlink_to("real/inert.h")
    _synthetic_q2_write(root, header_snapshot, header_bytes, 0o400)
    link_snapshot = root / "header-snapshots" / "inert-link"
    _synthetic_q2_write(root, link_snapshot, b"real/inert.h", 0o400)
    settings_snapshot = root / "header-snapshots" / "sdk-settings"
    _synthetic_q2_write(root, settings_snapshot, settings_path.read_bytes(), 0o400)

    compiler = _synthetic_q2_pin(root_pin, compiler_path, executable=True)
    linker = _synthetic_q2_pin(root_pin, linker_path, executable=True)
    runtime = _synthetic_q2_pin(root_pin, runtime_path)
    settings = _synthetic_q2_pin(root_pin, settings_path)
    header = _synthetic_q2_pin(root_pin, resolved_header)
    link_info = header_path.lstat()
    header.update({"path": str(header_path), "resolved_path": str(resolved_header), "snapshot": str(header_snapshot),
                   "symlinks": [{"path": str(header_path), "target": "real/inert.h", "len": len(b"real/inert.h"),
                                 "sha256": sha(b"real/inert.h"), "ctime_ns": link_info.st_ctime_ns,
                                 "dev": link_info.st_dev, "ino": link_info.st_ino, "mode": link_info.st_mode,
                                 "mtime_ns": link_info.st_mtime_ns, "nlink": link_info.st_nlink,
                                 "size": link_info.st_size, "uid": link_info.st_uid, "snapshot": str(link_snapshot)}]})
    settings_header = {**settings, "resolved_path": str(settings_path), "snapshot": str(settings_snapshot), "symlinks": []}
    flags = ["-target", "arm64-apple-darwin", "-isysroot", str(sdk_root), "-O2", "-v"]
    compiler.update({
        "declared_dylibs": str(compiler_path) + ":\n\t/usr/lib/libSystem.B.dylib (compatibility version 1.0.0, current version 1.0.0)\n",
        "flags": flags,
        "reported_target": "arm64-apple-darwin27.0.0",
        "target": "aarch64-apple-darwin",
        "version": "INERT compiler version claim; never executed",
    })

    # A header+LC_UUID only: thin parseable metadata, NO segments/code/entry.
    uuid = bytes.fromhex("12" * 16)
    executable_bytes = struct.pack("<8I", 0xFEEDFACF, 16_777_228, 0, 2, 1, 24, 0, 0)
    executable_bytes += struct.pack("<2I", 0x1B, 24) + uuid
    executable_path = root / "inert-probe-not-runnable"
    _synthetic_q2_write(root, executable_path, executable_bytes, 0o500)
    original_output = root / "build" / "inert-probe.original-not-runnable"
    _synthetic_q2_write(root, original_output, executable_bytes, 0o500)
    executable = _synthetic_q2_pin(root_pin, executable_path, executable=True)
    macho = {
        "cpu_type": 16_777_228, "cpu_subtype": 0, "uuid": uuid.hex(),
        "fat": False, "rpaths": [], "loads": [],
    }

    build_argv = [str(compiler_path), *flags, str(compiled_source), "-o", str(original_output)]
    # Exactly one quoted linker executable, followed by an unquoted archive
    # token. This tests the actual shlex-based closure parser without running it.
    build_stderr = '"' + str(linker_path) + '" -o ' + str(original_output) + " " + str(runtime_path) + "\n"
    build = {
        "argv": build_argv, "cwd": str(root), "exit_code": 0,
        "input": _synthetic_q2_pin(root_pin, compiled_source),
        "output": _synthetic_q2_pin(root_pin, original_output, executable=True),
        "stderr": build_stderr, "stderr_sha256": sha(build_stderr.encode()),
        "stdout": "", "stdout_sha256": sha(b""),
    }
    dep_flags = [flag for flag in flags if flag != "-v"]
    dep_stdout = "probe: " + " ".join((str(compiled_source), str(settings_path), str(header_path))) + "\n"
    dependency_discovery = {
        "argv": [str(compiler_path), *dep_flags, "-M", "-MT", "probe", str(compiled_source)],
        "exit_code": 0, "stdout": dep_stdout,
        "stdout_sha256": sha(dep_stdout.encode()),
    }
    for name, result, stream in (("build.stderr.log", build, "stderr"), ("build.stdout.log", build, "stdout"),
                                 ("dependencies.log", dependency_discovery, "stdout")):
        path = root / name
        _synthetic_q2_write(root, path, result[stream].encode(), 0o400)
        result["retained_" + stream] = _synthetic_q2_pin(root_pin, path)
    source_digest = hashlib.sha256(b"bboxy.dream.proof-tools/source/1\0")
    for name in ("resolve.rs", "trust.rs", "c1.rs", "read.rs"):
        content = (snapshot / "src/dream" / name).read_bytes()
        source_digest.update(name.encode() + b"\0" + len(content).to_bytes(8, "little") + content)
    receipt = {
        "basis": "conditional_source_review", "build": build,
        "candidate_run": False, "compiler": compiler,
        "created": "synthetic-unit-fixture-not-a-real-preparation",
        "dependency_discovery": dependency_discovery,
        "executable": executable, "format": "bboxy.dream.c-preparation/1",
        "header_inputs": {str(header_path): header, str(settings_path): settings_header}, "inputs": inputs,
        "input_commits": {name: preparation_commit if name in Q2_PREPARATION_NAMES else SYNTHETIC_Q2_COMMIT for name in Q2_INPUT_NAMES},
        "limitations": ["INERT input-ingestion-only fixture; no signature, compile, native or runtime evidence"],
        "linker": linker, "platform": dict(SYNTHETIC_Q2_PLATFORM),
        "private_test": "not_applicable_private_test",
        "provisioning_source": {
            "compiled_product_currentness_proven": False,
            "domain": r"bboxy.dream.proof-tools/source/1\0",
            "kind": "exact_source_snapshot", "sha256": source_digest.hexdigest(),
        },
        "runtime_approved": False,
        "schema_authority_commit": "b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9",
        "sdk": {"path": str(sdk_root), "settings": settings, "version": "27.0"},
        # Synthetic assertions below test binding only; no signing happens.
        "signature_verified": True, "signer_fingerprint": SYNTHETIC_Q2_SIGNER,
        "source_commit": SYNTHETIC_Q2_COMMIT, "source_snapshot": str(snapshot),
        "preparation_commit": preparation_commit, "preparation_signer_fingerprint": SYNTHETIC_Q2_SIGNER,
        "preparation_signature_verified": True, "preparation_snapshot": str(preparation_snapshot),
        "retained_root": str(root), "retained_directory_mode": "0700",
        "whole_command_lifetime_proof": False,
    }
    # Only immutable preparation outputs enter this synthetic inventory. The
    # inert toolchain/SDK stand in for external platform inputs, never outputs.
    output_files = [source_path, executable_path, compiled_source, original_output, header_snapshot, link_snapshot, settings_snapshot,
                    *(Path(build["retained_" + stream]["path"]) for stream in ("stdout", "stderr")),
                    Path(dependency_discovery["retained_stdout"]["path"]),
                    *((preparation_snapshot if name in Q2_PREPARATION_NAMES else snapshot) / name for name in Q2_INPUT_NAMES)]
    outputs = {}
    for path in output_files:
        outputs[str(path)] = {"kind": "file", **_synthetic_q2_pin(root_pin, path, executable=bool(path.stat().st_mode & 0o100))}
        for parent in path.parents:
            if not parent.is_relative_to(root): break
            info = parent.stat()
            outputs[str(parent)] = {"kind": "directory", "path": str(parent), "dev": info.st_dev,
                                    "ino": info.st_ino, "uid": info.st_uid, "mode": info.st_mode}
    receipt["retained_outputs"] = [outputs[name] for name in sorted(outputs)]
    receipt_bytes = json.dumps(
        receipt, sort_keys=True, separators=(",", ":"),
        ensure_ascii=False, allow_nan=False,
    ).encode() + b"\n"
    preparation_path = root / "synthetic-preparation.json"
    _synthetic_q2_write(root, preparation_path, receipt_bytes)
    preparation = _synthetic_q2_pin(root_pin, preparation_path)
    return {
        "root": root_pin, "source": source, "executable": executable,
        "preparation": preparation, "source_commit": SYNTHETIC_Q2_COMMIT,
        "signer_fingerprint": SYNTHETIC_Q2_SIGNER,
        "preparation_commit": preparation_commit, "preparation_signer_fingerprint": SYNTHETIC_Q2_SIGNER,
        "bindings": {name: q2_metadata_hash(receipt[name]) for name in Q2_BINDINGS.split()},
        "build_closure": [runtime], "macho": macho,
    }


class Q2InputAdapterIngestionTests(unittest.TestCase):
    """Focused inert-file unit cases; no native/qualification success claim."""

    def setUp(self):
        api = ingest_q2_input.__globals__
        self.selection_before = copy.deepcopy(api["ROOT_Q2_INPUT"])
        parent = Path(globals().get("TEST_TARGET", ROOT / "target/dream-proof-tools/q2-input-adapter"))
        self.assertTrue(parent.is_dir(), "parent must supply an existing private test destination")
        self.temporary = tempfile.TemporaryDirectory(prefix="ingestion-only-", dir=parent)
        self.addCleanup(self.temporary.cleanup)
        self.test_root = Path(self.temporary.name).resolve()
        self.fixture_number = 0

    def fixture(self, label="case"):
        self.fixture_number += 1
        root = self.test_root / (label + "-" + str(self.fixture_number))
        root.mkdir(mode=0o700)
        return synthetic_q2_input(root)

    @contextmanager
    def offline_only(self):
        api = ingest_q2_input.__globals__
        def forbidden(*args, **kwargs):
            raise AssertionError("input-ingestion unit test attempted execution/discovery/network")
        with ExitStack() as stack:
            for name in ("command", "invoke", "gpg", "q2_current_platform", "capture_dyld_context", "build", "component"):
                if name in api:
                    stack.enter_context(mock.patch.dict(api, {name: forbidden}))
            for name in ("run", "Popen", "call", "check_call", "check_output"):
                stack.enter_context(mock.patch.object(api["subprocess"], name, forbidden))
            for name in ("system", "fork", "posix_spawn", "posix_spawnp", "execve", "execv"):
                if hasattr(os, name):
                    stack.enter_context(mock.patch.object(os, name, forbidden))
            stack.enter_context(mock.patch.object(api["urllib"].request, "urlopen", forbidden))
            yield
        self.assertEqual(api["ROOT_Q2_INPUT"], self.selection_before)

    def ingest(self, selection):
        return ingest_q2_input(selection, current_platform=dict(SYNTHETIC_Q2_PLATFORM))

    def test_q2_input_inert_selection_ingests_without_launch_or_writes(self):
        selection = self.fixture("positive")
        root = Path(selection["root"]["path"])
        before = {str(path.relative_to(root)) for path in root.rglob("*")}
        with self.offline_only():
            result = self.ingest(selection)
        self.assertNotEqual(result["source"]["len"], 39_130)
        self.assertEqual(result["source"], selection["source"])
        self.assertEqual(result["executable"], selection["executable"])
        self.assertEqual(result["source_bytes"], Path(selection["source"]["path"]).read_bytes())
        self.assertEqual(result["scope"], "offline input ingestion only; no launch or runtime grant")
        self.assertEqual(before, {str(path.relative_to(root)) for path in root.rglob("*")})
        self.assertFalse(any(path.name in ("pretest-basis.json", "qualification.json", "receipt.candidate.json") for path in root.rglob("*")))

    def test_q2_input_exact_root_compiler_source_preserves_snapshot_binding(self):
        selection = self.fixture("compiler-source")
        receipt = json.loads(Path(selection["preparation"]["path"]).read_bytes())
        selection["source"] = dict(receipt["build"]["input"])
        with self.offline_only():
            result = self.ingest(selection)
        self.assertEqual(result["source"], receipt["build"]["input"])
        self.assertEqual(result["source_bytes"], (Path(receipt["source_snapshot"]) / Q2_C_SOURCE).read_bytes())
        # A byte-identical sibling is not the explicitly selected compiler input.
        foreign = Path(selection["root"]["path"]) / "unselected-source.c"
        _synthetic_q2_write(Path(selection["root"]["path"]), foreign, result["source_bytes"], 0o400)
        selection["source"] = _synthetic_q2_pin(selection["root"], foreign)
        with self.offline_only(), self.assertRaisesRegex(Refused, "snapshot/compiler input binding"):
            self.ingest(selection)

    def test_q2_input_root_source_must_match_complete_compiler_identity(self):
        selection = self.fixture("compiler-identity")
        receipt = json.loads(Path(selection["preparation"]["path"]).read_bytes())
        selection["source"] = dict(receipt["build"]["input"])
        original = Path(selection["source"]["path"])
        data = original.read_bytes()
        original.rename(original.with_suffix(".retained-original"))
        _synthetic_q2_write(Path(selection["root"]["path"]), original, data, 0o400)
        selection["source"] = _synthetic_q2_pin(selection["root"], original)
        with self.offline_only(), self.assertRaisesRegex(Refused, "snapshot/compiler input binding"):
            self.ingest(selection)

    def test_q2_input_missing_selection_refuses_before_any_private_read(self):
        api = ingest_q2_input.__globals__
        def forbidden_read(*args, **kwargs):
            raise AssertionError("missing selection reached filesystem input consumption")
        with self.offline_only(), mock.patch.dict(api, {"q2_private_file": forbidden_read, "ROOT_Q2_INPUT": None}):
            with self.assertRaisesRegex(Refused, "not selected by Root"):
                self.ingest(None)
            with self.assertRaisesRegex(Refused, "not selected by Root"):
                selected_q2_input()

    def test_q2_input_unset_acquisition_has_no_sessions_or_legacy_fallback(self):
        api = ingest_q2_input.__globals__
        def forbidden(*args, **kwargs):
            raise AssertionError("unset selection reached a file/session/native effect")
        replacements = {name: forbidden for name in ("session", "command", "invoke", "q2_private_file")}
        replacements["ROOT_Q2_INPUT"] = None
        with mock.patch.dict(api, replacements):
            for function, args in ((fetch, (self.test_root,)), (build, (self.test_root,)), (check_sources, (self.test_root,)),
                                   (component, (self.test_root,)), (prepare_basis, (self.test_root, self.test_root, {})),
                                   (source_lock, ()), (validate_proposal, (b"{}",))):
                with self.subTest(function=function.__name__), self.assertRaisesRegex(Refused, "not selected by Root"):
                    function(*args)

    def test_q2_input_replacement_during_held_read_refuses(self):
        selection = self.fixture("read-replacement")
        path = Path(selection["source"]["path"])
        content = path.read_bytes()
        original_read = os.read
        changed = False
        def replace_after_read(fd, count):
            nonlocal changed
            raw = original_read(fd, count)
            if raw and not changed:
                changed = True
                path.rename(path.with_name(path.name + ".retained-original"))
                _synthetic_q2_write(Path(selection["root"]["path"]), path, content)
            return raw
        with self.offline_only(), mock.patch.object(os, "read", replace_after_read), self.assertRaises(Refused):
            self.ingest(selection)
        self.assertTrue(changed)

    def test_q2_input_ambient_or_payload_claim_cannot_select_authority(self):
        selection = self.fixture("environment")
        poisoned = {"PATH": "/inert-untrusted", "HOME": "/inert-untrusted", "SDKROOT": "/inert-untrusted", "CC": "/inert-untrusted", "DYLD_INSERT_LIBRARIES": "/inert-untrusted", "ROOT_Q2_INPUT": "/inert-untrusted", "Q2_PROBE": "/inert-untrusted"}
        with self.offline_only(), mock.patch.dict(os.environ, poisoned):
            self.ingest(selection)
            with mock.patch.dict(ingest_q2_input.__globals__, {"ROOT_Q2_INPUT": None}), self.assertRaises(Refused):
                selected_q2_input()
            for key in ("approved", "runtime_approved", "canonical_q2"):
                with self.subTest(key=key), self.assertRaises(Refused):
                    self.ingest({**selection, key: True})

    def test_q2_input_source_executable_receipt_bytes_are_exact(self):
        for name in ("source", "executable", "preparation"):
            with self.subTest(name=name):
                selection = self.fixture("bytes-" + name)
                pin = selection[name]
                path = Path(pin["path"])
                old = path.stat()
                data = path.read_bytes()
                _synthetic_q2_overwrite(path, bytes([data[0] ^ 1]) + data[1:])
                os.utime(path, ns=(old.st_atime_ns, pin["mtime_ns"]))
                with self.offline_only(), self.assertRaises(Refused):
                    self.ingest(selection)

    def test_q2_input_same_bytes_replacement_does_not_inherit_identity(self):
        for name in ("source", "executable", "preparation"):
            with self.subTest(name=name):
                selection = self.fixture("inode-" + name)
                pin = selection[name]
                path = Path(pin["path"])
                data, mode = path.read_bytes(), stat.S_IMODE(path.stat().st_mode)
                retained = path.with_name(path.name + ".retained-original")
                path.rename(retained)
                _synthetic_q2_write(Path(selection["root"]["path"]), path, data, mode)
                os.utime(path, ns=(path.stat().st_atime_ns, pin["mtime_ns"]))
                self.assertNotEqual(path.stat().st_ino, retained.stat().st_ino)
                with self.offline_only(), self.assertRaises(Refused):
                    self.ingest(selection)

    def test_q2_input_each_offline_dependency_is_rechecked(self):
        for name in ("compiler", "linker", "sdk_settings", "header", "header_snapshot", "closure", "compiled_source", "compiled_output", "snapshot_rust"):
            with self.subTest(name=name):
                selection = self.fixture("dependency-" + name)
                receipt = _synthetic_q2_receipt(selection)
                header_name, header_pin = next(iter(receipt["header_inputs"].items()))
                paths = {
                    "compiler": receipt["compiler"]["path"],
                    "linker": receipt["linker"]["path"],
                    "sdk_settings": receipt["sdk"]["settings"]["path"],
                    "header": header_pin["resolved_path"], "header_snapshot": header_pin["snapshot"],
                    "closure": selection["build_closure"][0]["path"],
                    "compiled_source": receipt["build"]["argv"][-3],
                    "compiled_output": receipt["build"]["argv"][-1],
                    "snapshot_rust": str(Path(receipt["source_snapshot"]) / "src/dream/read.rs"),
                }
                path = Path(paths[name])
                old = path.stat()
                data = path.read_bytes()
                _synthetic_q2_overwrite(path, bytes([data[0] ^ 1]) + data[1:])
                os.utime(path, ns=(old.st_atime_ns, old.st_mtime_ns))
                with self.offline_only(), self.assertRaises(Refused):
                    self.ingest(selection)

    def test_q2_input_self_consistent_receipt_still_needs_inner_binding(self):
        for change in ("compiler_target", "sdk_settings_path", "header_snapshot", "missing_dependency", "unbound_header_alias", "missing_link_input", "source_claim", "runtime_claim", "source_commit", "signer"):
            with self.subTest(change=change):
                selection = self.fixture("inner-" + change)
                receipt = _synthetic_q2_receipt(selection)
                if change == "compiler_target":
                    receipt["compiler"]["reported_target"] = "x86_64-apple-darwin27.0.0"
                elif change == "sdk_settings_path":
                    receipt["sdk"]["settings"] = dict(receipt["linker"])
                elif change == "header_snapshot":
                    next(iter(receipt["header_inputs"].values()))["snapshot"] = receipt["build"]["argv"][-3]
                elif change == "missing_dependency":
                    header = next(iter(receipt["header_inputs"]))
                    receipt["dependency_discovery"]["stdout"] = receipt["dependency_discovery"]["stdout"].replace(" " + header, "")
                    receipt["dependency_discovery"]["stdout_sha256"] = sha(receipt["dependency_discovery"]["stdout"].encode())
                elif change == "unbound_header_alias":
                    header = next(iter(receipt["header_inputs"]))
                    alias = Path(header).with_name("inert-alias.h")
                    alias.symlink_to(header)
                    receipt["dependency_discovery"]["stdout"] = receipt["dependency_discovery"]["stdout"].replace(header, str(alias))
                    receipt["dependency_discovery"]["stdout_sha256"] = sha(receipt["dependency_discovery"]["stdout"].encode())
                elif change == "missing_link_input":
                    path = selection["build_closure"][0]["path"]
                    receipt["build"]["stderr"] = receipt["build"]["stderr"].replace(" " + path, "")
                    receipt["build"]["stderr_sha256"] = sha(receipt["build"]["stderr"].encode())
                elif change == "source_claim":
                    receipt["provisioning_source"]["compiled_product_currentness_proven"] = True
                elif change == "runtime_claim":
                    receipt["runtime_approved"] = True
                elif change == "source_commit":
                    receipt["source_commit"] = "2" * 40
                else:
                    receipt["signer_fingerprint"] = "B" * 40
                _synthetic_q2_reseal_receipt(selection, receipt)
                with self.offline_only(), self.assertRaises(Refused):
                    self.ingest(selection)

    def test_q2_input_macho_must_match_even_when_file_and_receipt_resealed(self):
        selection = self.fixture("macho")
        receipt = _synthetic_q2_receipt(selection)
        path = Path(selection["executable"]["path"])
        data = path.read_bytes()
        changed = data[:40] + bytes.fromhex("34" * 16)
        _synthetic_q2_overwrite(path, changed, 0o500)
        _synthetic_q2_overwrite(receipt["build"]["argv"][-1], changed, 0o500)
        pin = _synthetic_q2_pin(selection["root"], path, executable=True)
        selection["executable"] = pin
        receipt["executable"] = pin
        receipt["build"]["output"] = _synthetic_q2_pin(selection["root"], receipt["build"]["argv"][-1], executable=True)
        _synthetic_q2_reseal_receipt(selection, receipt)
        with self.offline_only(), self.assertRaisesRegex(Refused, "Mach-O facts mismatch"):
            self.ingest(selection)

    def test_q2_input_original_output_must_exist_and_match_supplied_copy(self):
        for change in ("missing", "unrelated"):
            with self.subTest(change=change):
                selection = self.fixture("original-output-" + change)
                receipt = _synthetic_q2_receipt(selection)
                original = Path(receipt["build"]["argv"][-1])
                self.assertNotEqual(str(original), selection["executable"]["path"])
                if change == "missing":
                    original.unlink()
                else:
                    data = original.read_bytes()
                    _synthetic_q2_overwrite(original, bytes([data[0] ^ 1]) + data[1:], 0o500)
                with self.offline_only(), self.assertRaises((Refused, OSError)):
                    self.ingest(selection)

    def test_q2_input_private_custody_refuses_links_modes_and_foreign_root(self):
        for change in ("leaf_symlink", "ancestor_symlink", "hardlink", "root_mode", "intermediate_mode", "source_mode", "executable_mode", "foreign_file", "root_identity"):
            with self.subTest(change=change):
                selection = self.fixture("custody-" + change)
                root = Path(selection["root"]["path"])
                source = Path(selection["source"]["path"])
                if change == "leaf_symlink":
                    retained = source.with_name(source.name + ".retained")
                    source.rename(retained)
                    source.symlink_to(retained)
                elif change == "ancestor_symlink":
                    retained = root / "retained-source-directory"
                    source.parent.rename(retained)
                    source.parent.symlink_to(retained, target_is_directory=True)
                elif change == "hardlink":
                    os.link(source, root / "extra-hardlink")
                elif change == "root_mode":
                    os.chmod(root, 0o755)
                elif change == "intermediate_mode":
                    os.chmod(source.parent, 0o755)
                elif change == "source_mode":
                    os.chmod(source, 0o644)
                elif change == "executable_mode":
                    os.chmod(selection["executable"]["path"], 0o600)
                elif change == "foreign_file":
                    outside = self.test_root / ("foreign-" + str(self.fixture_number))
                    _synthetic_q2_write(self.test_root, outside, b"INERT foreign owned input\n")
                    selection["source"]["path"] = str(outside)
                else:
                    selection["root"]["ino"] += 1
                with self.offline_only(), self.assertRaises((Refused, OSError)):
                    self.ingest(selection)

    def test_q2_input_changed_platform_is_offline_refusal(self):
        selection = self.fixture("platform")
        for key, value in (("machine", "x86_64"), ("release", "27.0.1"), ("os_build", "26A435")):
            with self.subTest(key=key), self.offline_only(), self.assertRaises(Refused):
                ingest_q2_input(selection, current_platform={**SYNTHETIC_Q2_PLATFORM, key: value})

    def test_q2_input_corrected_preparation_and_lexical_alias_bindings(self):
        selection = self.fixture("corrected-positive")
        receipt = _synthetic_q2_receipt(selection)
        self.assertNotEqual(receipt["source_commit"], receipt["preparation_commit"])
        alias = next(row for row in receipt["header_inputs"].values() if row["symlinks"])
        self.assertNotEqual(alias["path"], alias["resolved_path"])
        self.assertEqual(Path(alias["symlinks"][0]["snapshot"]).read_bytes(), b"real/inert.h")
        with self.offline_only(): self.ingest(selection)

    def test_q2_input_corrected_receipt_inner_controls(self):
        for change in ("preparation_commit", "preparation_signer", "preparation_signature", "input_commit", "dropped_output",
                       "directory_mode", "build_log", "symlink_target", "symlink_snapshot", "dropped_alias", "resolved_header"):
            with self.subTest(change=change):
                selection = self.fixture("corrected-" + change)
                receipt = _synthetic_q2_receipt(selection)
                alias = next(row for row in receipt["header_inputs"].values() if row["symlinks"])
                if change == "preparation_commit": receipt["preparation_commit"] = receipt["source_commit"]
                elif change == "preparation_signer": receipt["preparation_signer_fingerprint"] = "B" * 40
                elif change == "preparation_signature": receipt["preparation_signature_verified"] = False
                elif change == "input_commit": receipt["input_commits"][Q2_PREPARATION_NAMES[0]] = receipt["source_commit"]
                elif change == "dropped_output": receipt["retained_outputs"].pop()
                elif change == "directory_mode": receipt["retained_outputs"][0]["mode"] = stat.S_IFDIR | 0o755
                elif change == "build_log": receipt["build"]["retained_stderr"] = dict(receipt["build"]["retained_stdout"])
                elif change == "symlink_target": alias["symlinks"][0]["target"] = "foreign/inert.h"
                elif change == "symlink_snapshot": alias["symlinks"][0]["snapshot"] = alias["snapshot"]
                elif change == "dropped_alias": alias["symlinks"] = []
                else: alias["resolved_path"] = alias["path"]
                _synthetic_q2_reseal_receipt(selection, receipt)
                with self.offline_only(), self.assertRaises(Refused): self.ingest(selection)

    def test_q2_input_same_target_symlink_replacement_loses_identity(self):
        selection = self.fixture("alias-identity")
        receipt = _synthetic_q2_receipt(selection)
        alias = next(row for row in receipt["header_inputs"].values() if row["symlinks"])
        path = Path(alias["path"])
        path.rename(path.with_name("retained-original-alias"))
        path.symlink_to(alias["symlinks"][0]["target"])
        with self.offline_only(), self.assertRaises(Refused): self.ingest(selection)

    def test_q2_input_large_receipt_remains_offline_only(self):
        selection = self.fixture("large-offline")
        receipt = _synthetic_q2_receipt(selection)
        receipt["limitations"] = ["INERT offline provenance padding; not prepared input. " * 3_000]
        _synthetic_q2_reseal_receipt(selection, receipt)
        self.assertGreater(selection["preparation"]["len"], 131_072)
        self.assertLess(selection["preparation"]["len"], Q2_PREPARATION_LIMIT)
        with self.offline_only():
            result = self.ingest(selection)
        self.assertEqual(result["scope"], "offline input ingestion only; no launch or runtime grant")
        self.assertEqual(set(result), {"source", "source_bytes", "executable", "preparation", "proposal_source_sha256", "scope"})


class ProvisioningTests(unittest.TestCase):
    def test_closure_resolution_cannot_escape_then_reenter_or_disguise_candidate_as_cache(self):
        corpus = closure_authority_file("closure-boundary-vectors.json", CLOSURE_VECTORS_SHA, 8 * 1024 * 1024)
        vector = next(row for row in corpus["vectors"] if row["id"] == "valid-complete-candidate-graph")
        frozen = next(row["value"] for row in corpus["contexts"] if row["id"] == vector["context_id"])
        def fixture():
            return closure_strict(bytes.fromhex(vector["wire_hex"])), closure_json_decode(closure_metadata_bytes(frozen))
        for macro in ("@loader_path/", "@executable_path/"):
            value, context = fixture()
            value["images"][0]["macho"]["loads"][1]["name"] = macro + "../../codec-prefix/lib/private.dylib"
            context["image_views"] = value["images"]
            with self.assertRaisesRegex(Refused, "closure:load_resolution"):
                validate_closure(value, context)
        value, context = fixture()
        context["prefix"] = "/usr/lib/codec-prefix"
        forged = "/usr/lib/codec-prefix/lib/private.dylib"
        value["images"][0]["macho"]["loads"][0]["name"] = forged
        context["cache_names"].append(forged)
        context["image_views"] = value["images"]
        with self.assertRaisesRegex(Refused, "closure:cache_boundary"):
            validate_closure(value, context)
        value, context = fixture()
        name = "@loader_path/../lib//./private.dylib"
        value["images"][0]["macho"]["loads"][1]["name"] = name
        context["image_views"] = value["images"]
        self.assertEqual(validate_closure(value, context)["images"][0]["macho"]["loads"][1]["name"], name)
        with self.assertRaisesRegex(Refused, "closure:load_resolution"):
            closure_candidate_path("/private/codec-prefix", "bin", "")

    def test_signed_closure_codec_goldens(self):
        result = check_closure_vectors()
        self.assertEqual(result["count"], 324)
        self.assertEqual((result["accept"], result["reject"]), (44, 280))

    def test_macho_load_modes_and_genuine_repeated_commands(self):
        import struct
        commands = [struct.pack("<2I", 0x1b, 24) + bytes.fromhex("12" * 16)]
        modes = [(0xc, "load"), (0x80000018, "weak"), (0x8000001f, "reexport"), (0x80000023, "upward"), (0x20, "lazy"), (0xc, "load")]
        for cmd, _ in modes:
            name = b"/usr/lib/libSystem.B.dylib\0"
            size = (24 + len(name) + 7) // 8 * 8
            commands.append(struct.pack("<6I", cmd, size, 24, 0, 0, 0) + name + b"\0" * (size - 24 - len(name)))
        for _ in range(2):
            name = b"@loader_path/../lib\0"
            size = (12 + len(name) + 7) // 8 * 8
            commands.append(struct.pack("<3I", 0x8000001c, size, 12) + name + b"\0" * (size - 12 - len(name)))
        data = struct.pack("<8I", 0xfeedfacf, 16777228, 0, 2, len(commands), sum(map(len, commands)), 0, 0) + b"".join(commands)
        macho, install, _ = parse_macho(data, binary=True)
        self.assertIsNone(install)
        self.assertEqual([row["mode"] for row in macho["loads"]], [mode for _, mode in modes])
        self.assertEqual(macho["loads"][0], macho["loads"][-1])
        self.assertEqual(macho["rpaths"], ["@loader_path/../lib"] * 2)

    def test_closure_refuses_foreign_link_inputs_and_basename_source_claims(self):
        run = Path("/private/owned-build")
        sdk = Path("/Applications/bound-sdk")
        for tool in ("git", "gpg"):
            for entry in (b"/opt/homebrew/foreign.o", b"/opt/homebrew/foreign.tbd", b"unknown.input", b"../foreign.o", b"unreviewed/generated.o"):
                with self.assertRaises(Refused): review_link_map_input(run, tool, entry, sdk, {})
        for name in ("libgit.a", "libgcrypt.a", "libksba.a"):
            with self.assertRaises(Refused): closure_archive_source(run, run / "build/unreviewed" / name)
        self.assertEqual(closure_archive_source(run, run / "src/git-2.54.0/libgit.a"), "git")
        self.assertEqual(closure_archive_source(run, run / "build/deps/lib/libksba.a"), "libksba")

    def test_sdk_link_input_retains_umbrella_document_without_fabricated_edges(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as tmp:
            sdk = Path(tmp)
            stub = sdk / "libSystem.tbd"
            document = b"--- !tapi-tbd\ntbd-version: 4\ninstall-name: '/usr/lib/libSystem.B.dylib'\n...\n"
            embedded = b"--- !tapi-tbd\ntbd-version: 4\ninstall-name: '/usr/lib/system/libsystem_c.dylib'\n...\n"
            exclusive(stub, document + embedded)
            facts = review_link_map_input(Path("/unused"), "git", str(stub).encode(), sdk, {})
            self.assertEqual(facts["install_name"], "/usr/lib/libSystem.B.dylib")
            self.assertEqual(facts["documents"], 2)
            self.assertEqual(facts["sha256"], sha(document + embedded))
            stub.write_bytes(document.replace(b"...\n", b"install-name: '/usr/lib/ambiguous.dylib'\n...\n"))
            with self.assertRaises(Refused): review_link_map_input(Path("/unused"), "git", str(stub).encode(), sdk, {})

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

    def test_prepared_budget_charges_materialized_descriptor_before_launch(self):
        # Length-only regression from the retained failed source10 attempt;
        # these inert bytes are no provenance, authentication or native proof.
        basis, lock, result = b"b" * 47112, b"s" * 7176, b"r" * 65536
        prepared_budget(b"", lock, result, basis)
        prepared_budget(b"", lock, result, basis, b"d" * 4096)
        with self.assertRaises(Refused): prepared_budget(b"", lock, result, basis, b"d" * 16384)
        with self.assertRaises(Refused): prepared_budget(b"q" * 7153, lock, result, basis, b"d" * 4096)

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
        # The frozen old lock is historical source-pin test input only. It is
        # not selected by source_lock()/component() or a runtime authority.
        historical = strict_json(read_regular(FIX / "sources.lock.json", 32768), 32768)["canonical_q2"]
        value = source_lock_for_q2(historical)
        validate_lock(canonical(value), test_q2_tuple=historical)
        for key in ("archive_sha256", "signature_sha256", "public_key", "roots", "signed_bytes"):
            changed = json.loads(canonical(value))
            changed["sources"][0][key] = "forged"
            with self.assertRaises(Refused): validate_lock(canonical(changed), test_q2_tuple=historical)
        for key in ("profile_patches", "bootstrap_verifier", "canonical_q2", "backports"):
            changed = json.loads(canonical(value)); changed[key] = ["forged"]
            with self.assertRaises(Refused): validate_lock(canonical(changed), test_q2_tuple=historical)

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


class ComponentHandoffTests(unittest.TestCase):
    """Inert host predicate/wait seams; no native or source qualification."""

    def fixture(self, case="positive"):
        descriptor = {"run_id": "1" * 26, "nonce": "2" * 26, "incarnation": 1, "case": case,
                      "probe_source_sha256": "3" * 64, "probe_exe_sha256": "4" * 64}
        descriptor.update({key: {"sha256": "5" * 64} for key in
                           ("source_lock", "build", "source_review", "closure", "config", "commands", "git", "gpg")})
        result = {"kind": "private_candidate_result", "profile": "read-helper-native/1",
                  "runtime_approved": False, "availability": "not_applicable_private_test",
                  **{key: descriptor[key] for key in ("run_id", "nonce", "incarnation", "case")},
                  "input_sha256": sha(canonical(descriptor)),
                  "probe": {"source_sha256": descriptor["probe_source_sha256"], "exe_sha256": descriptor["probe_exe_sha256"]},
                  "binding": {key: descriptor[key]["sha256"] for key in
                              ("source_lock", "build", "source_review", "closure", "config", "commands", "git", "gpg")},
                  "basis": {"kind": "conditional_on_bound_source_review", "review_sha256": descriptor["source_review"]["sha256"], "whole_command_lifetime_proof": False},
                  "phase": "settled", "outcome": "expected_rejection" if case == "corrupt_proof" else "completed",
                  "observed_proof": "invalid" if case == "corrupt_proof" else "valid", "error": None,
                  "settlement": {"created": 16, "reaped": 16, "group_gone": True, "retired_coverage": True},
                  "metrics": {key: 1 for key in ("ordinary_max_ns", "controller_max_ns", "cleanup_max_ns", "native_cpu_max_ns",
                               "native_wall_max_ns", "inclusive_max_ns", "A_ns", "C_ns", "helper_group_rss_peak_bytes", "probe_rss_peak_bytes")}}
        result["metrics"].update({"Q_max_ns": None, "W_ns": None, "attempts": 16, "members_max": 2, "debt_charged_ns": 9, "debt_paid_ns": 9})
        return descriptor, result

    def test_component_total_wait4_cpu_replaces_retired_grant_quantum(self):
        descriptor, result = self.fixture()
        result["metrics"]["A_ns"] = 3000000000
        self.assertEqual(component_result(canonical(result) + b"\n", descriptor, 0), result)
        for value in (None, True, -1, 3000000001):
            with self.subTest(cpu=value), self.assertRaises(Refused):
                result["metrics"]["A_ns"] = value
                component_result(canonical(result) + b"\n", descriptor, 0)

    def test_component_process_create_wall_66331_limit_preserves_cpu_and_inclusive(self):
        descriptor, result = self.fixture()
        for wall in (34000000, 250000000):
            result["metrics"]["native_wall_max_ns"] = wall
            component_result(canonical(result) + b"\n", descriptor, 0)
        result["metrics"]["native_wall_max_ns"] = 251000000
        with self.assertRaises(Refused):
            component_result(canonical(result) + b"\n", descriptor, 0)
        result["metrics"]["native_wall_max_ns"] = 34000000
        for name, value in (("native_cpu_max_ns", 2000001), ("inclusive_max_ns", 3000001)):
            previous = result["metrics"][name]
            result["metrics"][name] = value
            with self.subTest(metric=name), self.assertRaises(Refused):
                component_result(canonical(result) + b"\n", descriptor, 0)
            result["metrics"][name] = previous

    def test_component_raw_verdict_keeps_lf_and_private_boundary(self):
        for case in ("positive", "corrupt_proof"):
            descriptor, result = self.fixture(case)
            raw = canonical(result) + b"\n"
            self.assertEqual(component_result(raw, descriptor, 0), result)
            with self.assertRaises(Refused): component_result(raw[:-1], descriptor, 0)
            with self.assertRaises(Refused): component_result(raw, descriptor, 81)

    def test_component_partial_stale_forged_and_unsettled_verdicts_refuse(self):
        for key in ("approval", "binding", "nonce", "proof", "settlement", "clock", "resource", "debt"):
            with self.subTest(key=key):
                descriptor, result = self.fixture()
                if key == "approval": result["runtime_approved"] = True
                elif key == "binding": result["binding"]["git"] = "6" * 64
                elif key == "nonce": result["nonce"] = "7" * 26
                elif key == "proof": result["observed_proof"] = "not_reached"
                elif key == "settlement": result["settlement"]["reaped"] = 15
                elif key == "clock": result["metrics"]["ordinary_max_ns"] = None
                elif key == "resource": result["metrics"]["inclusive_max_ns"] = 3000001
                else: result["metrics"]["debt_paid_ns"] = 0
                with self.assertRaises(Refused): component_result(canonical(result) + b"\n", descriptor, 0)

    def test_component_refusal_wire_does_not_exit_or_kill_owned_controller(self):
        # A mocked kernel wait seam distinguishes wire publication from actual
        # wait4. No child/process is born and no synthetic result is published.
        from types import SimpleNamespace
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as temp:
            folder = Path(temp)
            child = SimpleNamespace(pid=42, returncode=None)
            usage = SimpleNamespace(ru_utime=0, ru_stime=0, ru_maxrss=4096)
            sequence = [(0, 0, None), (42, 81 << 8, usage)]
            with mock.patch.object(os, "wait4", side_effect=sequence) as waited, mock.patch.object(time, "sleep"):
                result = wait_component(child, folder, 0)
            self.assertEqual(waited.call_count, 2)
            self.assertEqual(child.returncode, 81)
            self.assertTrue(result["actual_wait4"])
            pending = strict_json(read_regular(folder / "owned-pending.json", 32768, True), 32768)
            self.assertIs(pending["controller_reaped"], False)

    def test_component_unknown_spawn_or_refusal_wire_fences_replacement(self):
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as temp:
            run = Path(temp)
            private_dir(run / "logs"); private_dir(run / "components")
            case = private_dir(run / "components/cancel")
            exclusive(case / "spawn-intent.json", b"{}")
            exclusive(case / "result.json", b'{"outcome":"quarantined"}\n')
            self.assertEqual(pending_commands(run), [case / "spawn-intent.json"])
            exclusive(case / "wait4.json", b'{"actual_wait4":true}')
            self.assertEqual(pending_commands(run), [])

    def test_descriptor_uses_c_controller_clock_not_python_mach_clock(self):
        # Inert file-pin construction only: no decoder, helper or fake proof.
        # Darwin Python monotonic_ns and C CLOCK_MONOTONIC have distinct
        # epochs/rates; the retained real refusal measured a 2.806s offset.
        with tempfile.TemporaryDirectory(dir=TEST_TARGET) as temp:
            root = Path(temp)
            prefix = private_dir(root / "candidate"); private_dir(prefix / "bin"); private_dir(prefix / "provenance")
            repo = private_dir(root / "fixture"); output = private_dir(root / "output")
            for path in (prefix / "bin/git", prefix / "bin/gpg", prefix / "provenance/pretest-basis.json", prefix / "provenance/lock", repo / "public.asc"):
                exclusive(path, b"inert unit bytes; never executed")
            lock_bytes = read_regular(prefix / "provenance/lock", 32768, True)
            basis = {"build": {"incarnation": 1, "platform": {"os_build": "26A434"},
                               "source_lock": {"path": "provenance/lock", "len": len(lock_bytes), "sha256": sha(lock_bytes)}},
                     "fixture": {"repo": dir_pin(repo), "valid_commit": "1" * 40, "corrupt_commit": "2" * 40, "public_keys": {"path": "public.asc"}}}
            selection = {"source": {"sha256": "3" * 64}, "executable": {"sha256": "4" * 64}}
            with mock.patch.object(time, "clock_gettime_ns", return_value=100) as c_clock, mock.patch.object(time, "monotonic_ns", return_value=10**18):
                descriptor, _ = private_descriptor(root, prefix, basis, "positive", output, selection)
            self.assertEqual(descriptor["deadline_ns"], 30_000_000_100)
            c_clock.assert_called_once_with(time.CLOCK_MONOTONIC)

    def test_cached_shared_cache_identity_is_rechecked_even_with_same_os(self):
        platform_pin = {"shared_cache_uuid": "1" * 32, "os_build": "26A434", "release": "27.0.0", "arch": "aarch64"}
        basis = {"build": {"platform": dict(platform_pin)}, "closure": {"platform": dict(platform_pin)}}
        with mock.patch.dict(basis_platform_current.__globals__, {"current_cache_uuid": lambda: "1" * 32}):
            basis_platform_current(basis, "1" * 32)
            with self.assertRaises(Refused): basis_platform_current(basis, "2" * 32)
        with mock.patch.dict(basis_platform_current.__globals__, {"current_cache_uuid": lambda: "2" * 32}):
            with self.assertRaises(Refused): basis_platform_current(basis)

    def terminal_fixture(self, case="positive", extra_errors=None):
        # Inert copies of the selected producer's exact raw diagnostic shapes;
        # these do not execute Q2 or provide qualification/native evidence.
        codes = ("cancelled", "clock_unknown", "coverage_unknown", "deadline", "input_invalid", "resource_limit",
                 "review_unbound", "settlement_unknown", "signature_unexpected", "source_changed", "other")
        causal = {"cancel": "cancelled", "stale_incarnation": "coverage_unknown", "clock_failure": "clock_unknown",
                  "debt_overlap": "coverage_unknown", "reap_delay": "settlement_unknown"}
        good = case in ("positive", "corrupt_proof", "pause_resume")
        errors = {}
        if not good:
            errors[causal[case]] = {"code": causal[case], "count": 1, "details_truncated": False,
                                  "first_detail": "INERT causal-control diagnostic", "last_detail": "INERT causal-control diagnostic",
                                  "kind": "private_candidate_error"}
        if extra_errors: errors.update(extra_errors)
        counts = {code: errors[code]["count"] if code in errors else 0 for code in codes}
        row = {"profile": "read-helper-native/1", "stage": "settled" if good else "settling",
               "metrics_cutoff": "pre_result_emission", "metadata_kind": "declared_peak_capacity",
               "terminal_cutoff": "after_paid_tail_and_error_diagnostics_before_final_receipt", "runtime_approved": "false",
               "helper_created": "16" if good else "1", "helper_reaped": "16" if good else "1", "result_complete": "true",
               "control_verdict": "passed" if good else "failed", "persistence_verdict": "persisted", "terminal_cost_verdict": "passed",
               "tail_complete": "true", "persistence_attempted": "true", "persistence_open": "true", "persistence_write": "true",
               "persistence_fsync": "true", "persistence_close": "true", "persistence_directory_fsync": "true", "persistence_errno": "0",
               "stdout_errno": "0", "stdout_attempted": "true", "reporting_only": "1", "original_entry_ns": "1",
               "original_deadline_ns": "30000000001", "deadline_observed_ns": "0", "deadline_first_ns": "0",
               "deadline_first_unit_ns": "0", "deadline_first_line": "0", "deadline_last_line": "0",
               "reporting_started_ns": "2", "closed_unit_count": "1", "path_walk_components": "1", "path_walk_units": "1",
               "booked_ordinary_units": "3", "engine_charge_calls": "1", "poll_closure_calls": "1",
               "ordinary_cost_ns": "3", "ordinary_raw_cpu_ns": "3", "ordinary_raw_complete": "true",
               "ordinary_floor_surcharge_ns": "0", "helper_cost_total_ns": "0", "debt_components_match": "true",
               "debt_scope": "whole_controller_cumulative",
               "process_create_wall_excess_ns": "0", "process_create_wall_excess_count": "0",
               "process_create_wall_excess_known": "1",
               "process_create_wall_excess_scope": "invocation_sum_once_per_prepared_spawn",
               "persistence_paid": "true", "formatter_cpu_start_ns": "1", "formatter_cpu_end_ns": "2",
               "formatter_inclusive_ns": "1", "write_cpu_start_ns": "3", "write_cpu_end_ns": "4", "write_inclusive_ns": "1",
               "close_max_cpu_ns": "1", "probe_rss_peak_bytes": "4096", "rss_known": "true", "ordinary_max_ns": "1",
               "debt_charged_ns": "27", "debt_paid_ns": "27", "retained_units": "0", "retained_overflow": "0", "format_paid": "true",
               "write_paid": "true", "error_count": str(sum(counts.values())), "error_codes": ",".join(codes),
               "error_counts": ",".join(str(counts[code]) for code in codes), "errors_complete": "true", "diagnostics_complete": "true",
               "parent_actual_reap_required": "true"}
        end = {"output_closed": "true", "close_max_ns": "1", "debt_charged_ns": "9", "debt_paid_ns": "9", "ordinary_max_ns": "1"}
        def raw():
            tail = "private_candidate_exit_tail " + " ".join(k + "=" + v for k, v in end.items()) + "\n"
            diagnostics = b"".join(canonical(errors[code]) + b"\n" for code in codes if code in errors)
            final = "private_candidate_terminal " + " ".join(k + "=" + v for k, v in row.items()) + "\n"
            return tail.encode() + diagnostics + final.encode()
        return row, end, errors, raw

    def test_component_independent_terminal_verdicts_accept_only_complete_evidence(self):
        for case in ("positive", "corrupt_proof", "pause_resume", "cancel", "stale_incarnation", "clock_failure", "debt_overlap", "reap_delay"):
            with self.subTest(case=case):
                row, _, errors, raw = self.terminal_fixture(case)
                actual_exit = 0 if case in ("positive", "corrupt_proof") else 81
                observed = component_terminal(raw(), case, actual_exit=actual_exit)
                self.assertEqual(observed["terminal"], row)
                self.assertEqual(observed["errors"], errors)
                with self.assertRaises(Refused): component_terminal(raw(), case, actual_exit=0 if actual_exit else 81)

    def test_component_result_path_terminal_fields_are_required_and_closed(self):
        row, _, _, raw = self.terminal_fixture()
        component_terminal(raw(), "positive", actual_exit=0)
        names = ("stdout_attempted", "reporting_only", "original_entry_ns", "original_deadline_ns",
                 "deadline_observed_ns", "deadline_first_ns", "deadline_first_unit_ns", "deadline_first_line",
                 "deadline_last_line", "reporting_started_ns", "closed_unit_count", "path_walk_components", "path_walk_units")
        for name in names:
            with self.subTest(missing=name):
                value = row.pop(name)
                with self.assertRaisesRegex(Refused, "unknown/missing fields"):
                    component_terminal(raw(), "positive", actual_exit=0)
                row[name] = value
        row["future_terminal_field"] = "true"
        with self.assertRaisesRegex(Refused, "unknown/missing fields"):
            component_terminal(raw(), "positive", actual_exit=0)

    def test_component_result_path_terminal_types_and_bounds(self):
        # Inert wire parsing only; field widths and sentinels come from the
        # frozen C declarations, not a new controller or runtime qualification.
        limits = {name: 2**64 - 1 for name in ("original_entry_ns", "original_deadline_ns", "deadline_observed_ns",
                  "deadline_first_ns", "reporting_started_ns", "closed_unit_count", "path_walk_components", "path_walk_units",
                  "booked_ordinary_units", "engine_charge_calls", "poll_closure_calls", "ordinary_cost_ns",
                  "ordinary_raw_cpu_ns", "ordinary_floor_surcharge_ns", "helper_cost_total_ns")}
        limits.update({"deadline_first_line": 2**32 - 1, "deadline_last_line": 2**32 - 1,
                       "deadline_first_unit_ns": 1000000})
        for name, maximum in limits.items():
            row, _, _, raw = self.terminal_fixture()
            for value in ("0", str(maximum)):
                with self.subTest(name=name, accepted=value):
                    row[name] = value
                    component_terminal(raw(), "positive", actual_exit=0)
            for value in ("-1", "-0", "01", "1.0", "1e0", "true", "+1", str(maximum + 1)):
                with self.subTest(name=name, rejected=value), self.assertRaises(Refused):
                    row[name] = value
                    component_terminal(raw(), "positive", actual_exit=0)
        for name, accepted, rejected in (("stdout_attempted", ("true", "false"), ("0", "1", "True", "null")),
                                          ("reporting_only", ("1",), ("0", "2", "01", "true", "-1", "4294967296"))):
            row, _, _, raw = self.terminal_fixture()
            for value in accepted:
                row[name] = value
                component_terminal(raw(), "positive", actual_exit=0)
            for value in rejected:
                with self.subTest(name=name, rejected=value), self.assertRaises(Refused):
                    row[name] = value
                    component_terminal(raw(), "positive", actual_exit=0)

    def test_component_input54_terminal_fields_are_required_closed_and_typed(self):
        row, _, _, raw = self.terminal_fixture()
        names = ("booked_ordinary_units", "engine_charge_calls", "poll_closure_calls", "ordinary_cost_ns",
                 "ordinary_raw_cpu_ns", "ordinary_floor_surcharge_ns", "helper_cost_total_ns",
                 "ordinary_raw_complete", "debt_components_match", "debt_scope")
        for name in names:
            with self.subTest(missing=name):
                value = row.pop(name)
                with self.assertRaisesRegex(Refused, "unknown/missing fields"):
                    component_terminal(raw(), "positive", actual_exit=0)
                row[name] = value
            with self.subTest(duplicate=name), self.assertRaisesRegex(Refused, "duplicate token"):
                duplicate = raw()[:-1] + (" " + name + "=" + row[name] + "\n").encode()
                component_terminal(duplicate, "positive", actual_exit=0)
        row["input54_future_field"] = "0"
        with self.assertRaisesRegex(Refused, "unknown/missing fields"):
            component_terminal(raw(), "positive", actual_exit=0)
        del row["input54_future_field"]
        for name in ("ordinary_raw_complete", "debt_components_match"):
            for value in ("true", "false"):
                row[name] = value
                component_terminal(raw(), "positive", actual_exit=0)
            for value in ("0", "1", "True", "null", ""):
                with self.subTest(boolean=name, rejected=value), self.assertRaises(Refused):
                    row[name] = value
                    component_terminal(raw(), "positive", actual_exit=0)
            row[name] = "true"
        for value in ("", "whole_helper_cumulative", "whole_controller", "true"):
            with self.subTest(scope=value), self.assertRaisesRegex(Refused, "INPUT54 debt scope"):
                row["debt_scope"] = value
                component_terminal(raw(), "positive", actual_exit=0)

    def test_component_input60_wall_excess_u64(self):
        for name in ("process_create_wall_excess_ns", "process_create_wall_excess_count"):
            row, _, _, raw = self.terminal_fixture()
            for value in ("0", str(2**64 - 1)):
                row[name] = value
                component_terminal(raw(), "positive", actual_exit=0)
            for value in ("-1", "-0", "01", "1.0", "1e0", "true", str(2**64)):
                with self.subTest(field=name, rejected=value), self.assertRaises(Refused):
                    row[name] = value
                    component_terminal(raw(), "positive", actual_exit=0)

    def test_component_input60_wall_excess_known_numeric_flag(self):
        row, _, _, raw = self.terminal_fixture()
        for value in ("0", "1"):
            row["process_create_wall_excess_known"] = value
            component_terminal(raw(), "positive", actual_exit=0)
        for value in ("2", "true", "false", "01", "-1", "1.0", str(2**32)):
            with self.subTest(value=value), self.assertRaises(Refused):
                row["process_create_wall_excess_known"] = value
                component_terminal(raw(), "positive", actual_exit=0)

    def test_component_input60_wall_excess_scope_and_closed_fields(self):
        row, _, _, raw = self.terminal_fixture()
        names = ("process_create_wall_excess_ns", "process_create_wall_excess_count",
                 "process_create_wall_excess_known", "process_create_wall_excess_scope")
        for name in names:
            value = row.pop(name)
            with self.subTest(missing=name), self.assertRaisesRegex(Refused, "unknown/missing fields"):
                component_terminal(raw(), "positive", actual_exit=0)
            row[name] = value
            with self.subTest(duplicate=name), self.assertRaisesRegex(Refused, "duplicate token"):
                component_terminal(raw()[:-1] + (" " + name + "=" + value + "\n").encode(), "positive", actual_exit=0)
        row["input60_future_field"] = "0"
        with self.assertRaisesRegex(Refused, "unknown/missing fields"):
            component_terminal(raw(), "positive", actual_exit=0)
        del row["input60_future_field"]
        for value in ("", "original_prepared_spawn_snapshot", "whole_controller_cumulative", "true"):
            with self.subTest(scope=value), self.assertRaisesRegex(Refused, "process-create excess scope"):
                row["process_create_wall_excess_scope"] = value
                component_terminal(raw(), "positive", actual_exit=0)

    def test_component_terminal_failure_cannot_be_hidden_by_expected_control_exit(self):
        mutations = {"control_verdict": "unknown", "persistence_verdict": "unknown", "terminal_cost_verdict": "failed",
                     "tail_complete": "false", "persistence_write": "false", "persistence_fsync": "false", "persistence_close": "false",
                     "persistence_directory_fsync": "false", "persistence_errno": "5", "stdout_errno": "5", "persistence_paid": "false",
                     "format_paid": "false", "write_paid": "false", "write_inclusive_ns": "1000001", "retained_units": "1",
                     "retained_overflow": "1", "debt_paid_ns": "26", "errors_complete": "false", "diagnostics_complete": "false",
                     "parent_actual_reap_required": "false", "helper_reaped": "0", "rss_known": "false", "error_count": "2"}
        for key, value in mutations.items():
            with self.subTest(key=key):
                row, _, _, raw = self.terminal_fixture("cancel")
                row[key] = value
                with self.assertRaises(Refused): component_terminal(raw(), "cancel", actual_exit=81)

    def test_component_final_receipt_follows_paid_tail_and_complete_error_rows(self):
        _, _, _, make_raw = self.terminal_fixture("cancel")
        raw = make_raw(); lines = raw.splitlines(keepends=True)
        controls = (b"".join((lines[-1], *lines[:-1])), b"".join((lines[0], lines[-1])),
                    raw + lines[-1], raw + b"private trailing failure\n", raw[:-1],
                    raw.replace(b" helper_reaped=1 ", b" helper_reaped=1 helper_reaped=1 "),
                    raw.replace(b"control_verdict=failed", b"control_verdict=passed"),
                    raw.replace(b"terminal_cutoff=after_paid_tail_and_error_diagnostics_before_final_receipt", b"terminal_cutoff=pre_terminal_receipt"))
        for index, value in enumerate(controls):
            with self.subTest(index=index), self.assertRaises(Refused): component_terminal(value, "cancel", actual_exit=81)

    def test_component_error_histogram_preserves_later_failures(self):
        extra = {"resource_limit": {"code": "resource_limit", "count": 1, "details_truncated": False,
                                   "first_detail": "ordinary validation exceeds one millisecond", "last_detail": "ordinary validation exceeds one millisecond",
                                   "kind": "private_candidate_error"}}
        _, _, _, raw = self.terminal_fixture("cancel", extra)
        with self.assertRaisesRegex(Refused, "additional error classes"): component_terminal(raw(), "cancel", actual_exit=81)
        row, _, errors, raw = self.terminal_fixture("cancel")
        errors["cancelled"]["count"] = 2
        with self.assertRaisesRegex(Refused, "disagree with final histogram"): component_terminal(raw(), "cancel", actual_exit=81)
        errors["cancelled"]["count"] = 1; errors["cancelled"]["details_truncated"] = True
        with self.assertRaisesRegex(Refused, "truncated"): component_terminal(raw(), "cancel", actual_exit=81)
        errors["cancelled"]["details_truncated"] = False; row["error_counts"] = "01," + row["error_counts"].split(",", 1)[1]
        with self.assertRaisesRegex(Refused, "integer"): component_terminal(raw(), "cancel", actual_exit=81)

    def test_component_terminal_result_and_actual_settlement_are_cross_bound(self):
        descriptor, result = self.fixture()
        _, _, _, raw = self.terminal_fixture()
        checked = component_result(canonical(result) + b"\n", descriptor, 0)
        component_terminal(raw(), "positive", checked, 0)
        checked["settlement"]["created"] = 15
        with self.assertRaisesRegex(Refused, "settlement/result binding"): component_terminal(raw(), "positive", checked, 0)

    def test_component_source_known_auxiliary_errors_do_not_waive_terminal_gates(self):
        detail = "process creation native2ms wall250ms application1ms inclusive3ms gate"
        extra = {"resource_limit": {"code": "resource_limit", "count": 1, "details_truncated": False,
                                   "first_detail": detail, "last_detail": detail, "kind": "private_candidate_error"}}
        row, _, errors, raw = self.terminal_fixture("clock_failure", extra)
        component_terminal(raw(), "clock_failure", actual_exit=81)
        errors["resource_limit"]["last_detail"] = "unrelated resource failure"
        with self.assertRaisesRegex(Refused, "additional resource failure"): component_terminal(raw(), "clock_failure", actual_exit=81)
        errors["resource_limit"]["last_detail"] = detail; row["terminal_cost_verdict"] = "unknown"
        with self.assertRaises(Refused): component_terminal(raw(), "clock_failure", actual_exit=81)

    def test_clock_control_refuses_retired_or_changed_native_limit_details(self):
        for detail in (
            "unchanged native2ms wall25ms application1ms inclusive3ms gate",
            "process creation native2ms wall25ms application1ms inclusive3ms gate",
            "process creation native3ms wall250ms application1ms inclusive3ms gate",
            "process creation native2ms wall250ms application1ms inclusive4ms gate",
        ):
            with self.subTest(detail=detail):
                extra = {"resource_limit": {"code": "resource_limit", "count": 1, "details_truncated": False,
                                           "first_detail": detail, "last_detail": detail, "kind": "private_candidate_error"}}
                _, _, _, raw = self.terminal_fixture("clock_failure", extra)
                with self.assertRaisesRegex(Refused, "additional resource failure"):
                    component_terminal(raw(), "clock_failure", actual_exit=81)

    def test_reap_delay_accepts_only_original_partial_snapshot_and_actual_later_settlement(self):
        row, _, _, raw = self.terminal_fixture("reap_delay")
        row["stage"] = "settled"
        result = {"phase": "quarantined", "outcome": "quarantined", "observed_proof": "not_reached",
                  "metrics": {"attempts": 1}, "error": {"code": "settlement_unknown"},
                  "settlement": {"created": 1, "reaped": 0, "group_gone": False, "retired_coverage": False}}
        before = canonical(result)
        component_terminal(raw(), "reap_delay", result, 81)
        self.assertEqual(canonical(result), before)
        for actual_exit in (None, 0):
            with self.subTest(actual_exit=actual_exit), self.assertRaises(Refused):
                component_terminal(raw(), "reap_delay", result, actual_exit)
        for path, value in (("phase", "refused"), ("outcome", "failed"), ("observed_proof", "valid"),
                            ("metrics.attempts", 2), ("metrics.attempts", True),
                            ("settlement.reaped", 1), ("settlement.group_gone", True),
                            ("settlement.retired_coverage", True), ("settlement.group_gone", 0),
                            ("error.code", "clock_unknown")):
            changed = copy.deepcopy(result); parent = changed; keys = path.split(".")
            for key in keys[:-1]: parent = parent[key]
            parent[keys[-1]] = value
            with self.subTest(path=path, value=value), self.assertRaises(Refused):
                component_terminal(raw(), "reap_delay", changed, 81)
        row["helper_reaped"] = "0"
        with self.assertRaises(Refused): component_terminal(raw(), "reap_delay", result, 81)
        row["helper_reaped"] = "1"; row["terminal_cost_verdict"] = "unknown"
        with self.assertRaises(Refused): component_terminal(raw(), "reap_delay", result, 81)
        row, _, _, raw = self.terminal_fixture("cancel"); row["stage"] = "settled"
        changed = copy.deepcopy(result); changed["error"]["code"] = "cancelled"
        with self.assertRaises(Refused): component_terminal(raw(), "cancel", changed, 81)


def unit(d, signature_only=False):
    global TEST_TARGET
    TEST_TARGET = private_dir(d / "unit")
    suite = unittest.defaultTestLoader.loadTestsFromTestCase(ProvisioningTests)
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(Q2InputAdapterIngestionTests))
    suite.addTests(unittest.defaultTestLoader.loadTestsFromTestCase(ComponentHandoffTests))
    if signature_only:
        suite = unittest.TestSuite(t for t in suite if t._testMethodName.startswith("test_git_signature_"))
    result = unittest.TextTestRunner(verbosity=2).run(suite)
    require(result.wasSuccessful(), "private provisioning regression failed")


def check(d, component_only=False, source_only=False, signature_corpus_only=False, closure_only=False, closure_projection_only=False):
    if closure_only:
        result = check_closure_vectors()
        run = session(d, "closure-codec")
        exclusive(run / "closure-codec.json", canonical({"source_sha256": sha(SCRIPT_BYTES), **result}))
        print(f"C codec controls count={result['count']} accept={result['accept']} reject={result['reject']} retained at {run}; synthetic only", flush=True)
        return result
    if closure_projection_only:
        return check_closure_projection(d)
    if signature_corpus_only:
        return check_signature_corpus(d)
    if source_only:
        return check_sources(d)
    if not component_only:
        unit(d)
    result = component(d)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=["fetch", "build", "check", "unit"])
    parser.add_argument("--target", type=Path, default=ROOT / "target")
    parser.add_argument("--component-only", action="store_true")
    parser.add_argument("--source-only", action="store_true")
    parser.add_argument("--signature-corpus-only", action="store_true")
    parser.add_argument("--closure-only", action="store_true")
    parser.add_argument("--closure-projection-only", action="store_true")
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
            require(sum((args.component_only, args.source_only, args.signature_corpus_only, args.closure_only, args.closure_projection_only)) <= 1, "choose one private focused check")
            check(d, args.component_only, args.source_only, args.signature_corpus_only, args.closure_only, args.closure_projection_only)
    except (Refused, OSError, subprocess.TimeoutExpired) as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
