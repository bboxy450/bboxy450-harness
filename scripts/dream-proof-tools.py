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
        require(len(data) == s.st_size and (s.st_dev, s.st_ino, s.st_mtime_ns, s.st_size) == (t.st_dev, t.st_ino, t.st_mtime_ns, t.st_size), f"changed read: {path}")
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


def invoke(argv, home, *, data=None, cwd=None, timeout=600):
    result = subprocess.run(argv, input=data, cwd=cwd, env=env(home), stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False)
    return result


def gpg(home, args, data=None):
    return invoke([VERIFIER, "--no-options", "--batch", "--no-tty", "--no-autostart", "--no-auto-key-retrieve", "--no-auto-key-import", "--auto-key-locate", "clear", "--no-auto-check-trustdb", *args], home, data=data)


def pinned(data, expected, label):
    require(sha(data) == expected, f"hash mismatch: {label}: actual {sha(data)} expected {expected}")
    return data


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
    exclusive(run / "identity.json", canonical({"build_id": build_id, "incarnation": time.monotonic_ns(), "kind": kind, "pid": os.getpid(), "created_ns": time.time_ns()}))
    exclusive(run / "recipe.py", SCRIPT_BYTES)
    return run


def fetch(d):
    run = session(d, "fetch")
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
        archive = pinned(read_regular(cache / "archive", MAX_ARCHIVE, True), digest, name) if (cache / "archive").exists() else pinned(download(url), digest, name)
        sigurl = url[:-3] + ".sign" if name == "git" else url + ".sig"
        sigfile = FIX / (name + ".sig")
        signature = pinned(read_regular(sigfile, MAX_ARCHIVE), sigsha, name + " signature") if sigfile.exists() else pinned(download(sigurl), sigsha, name + " signature")
        if not sigfile.exists():
            exclusive(sigfile, signature)
        # A cached auth.json never substitutes for fresh crypto verification.
        record, raw = authenticate(source, archive, signature, keys["git" if name == "git" else "gnupg"], home)
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
    run = session(d, "build")
    home = private_dir(run / "public-home")
    src = private_dir(run / "src")
    private_dir(run / "build")
    private_dir(run / "logs")
    candidate = private_dir(d / "candidates" / run.name)
    validate_lock(read_regular(FIX / "sources.lock.json", 32768))
    exclusive(run / "sources.lock.json", read_regular(FIX / "sources.lock.json", 32768))
    records = []
    for source in PINS:
        cache = d / "cache" / source[3]
        archive = read_regular(cache / "archive", MAX_ARCHIVE, True)
        signature = read_regular(cache / "signature", MAX_ARCHIVE, True)
        key = read_regular(FIX / (("git" if source[0] == "git" else "gnupg") + "-release-key.asc"), MAX_ARCHIVE)
        record, raw = authenticate(source, archive, signature, key, home)
        records.append(record)
        extract(archive, "xz" if source[0] == "git" else "bz2", src, source[0] + "-" + source[1])
    exclusive(run / "authenticated.json", canonical(records))
    apply_profile(run)
    print(f"authenticated sources extracted: {run}; candidate prefix: {candidate}", flush=True)
    compile_sources(run, candidate)


def command(run, argv, *, cwd=None, extra_env=None, allow_failure=False):
    logs = run / "logs"
    seq = len(list(logs.glob("*.command.json")))
    stem = logs / f"{seq:03d}"
    effective = env(run / "public-home")
    effective.update(extra_env or {})
    request = {"argv": argv, "cwd": str(cwd or run), "env": effective, "source_sha256": sha(SCRIPT_BYTES)}
    exclusive(stem.with_suffix(".command.json"), canonical(request))
    # Every build is serial at this owner; make itself gets exactly two niced
    # jobs. A lost process outcome leaves this start record without an exit.
    with open(stem.with_suffix(".stdout"), "xb", opener=lambda p, flags: os.open(p, flags, 0o600)) as out, open(stem.with_suffix(".stderr"), "xb", opener=lambda p, flags: os.open(p, flags, 0o600)) as err:
        child = subprocess.Popen(["/usr/bin/nice", "-n", "10", *argv], cwd=cwd or run, env=effective, stdout=out, stderr=err)
        exclusive(stem.with_suffix(".pid.json"), canonical({"pid": child.pid, "operation": stem.name, "build_id": run.name}))
        try:
            status = child.wait(timeout=1200)
        except subprocess.TimeoutExpired:
            child.terminate()
            try:
                child.wait(timeout=5)
            except subprocess.TimeoutExpired:
                child.kill()
                child.wait()
            exclusive(stem.with_suffix(".timeout.json"), canonical({"pid": child.pid, "exit_code": child.returncode}))
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
    command(run, ["/usr/bin/xcrun", "--find", "ld"])
    command(run, ["/usr/bin/xcrun", "--show-sdk-version"])
    command(run, ["/usr/bin/sw_vers", "-buildVersion"])
    exclusive(run / "platform.json", canonical({"compiler": compiler, "compiler_sha256": sha(read_regular(Path(compiler).resolve())), "sdk": sdk, "sdk_settings_sha256": sha(read_regular(Path(sdk) / "SDKSettings.json")), "os_release": platform.release(), "arch": platform.machine(), "trust_assumptions": ["independent bootstrap verifier", "compiler/linker and SDK fidelity", "native platform/libSystem implementation; no arbitrary native theorem"]}))
    deps = private_dir(run / "build" / "deps")
    buildenv = {"CC": compiler, "SDKROOT": sdk, "CFLAGS_FOR_BUILD": "-O2 -isysroot " + sdk, "CFLAGS": "-O2 -g0 -isysroot " + sdk, "CPPFLAGS": "-isysroot " + sdk + " -I" + str(deps / "include"), "LDFLAGS": "-L" + str(deps / "lib") + " -isysroot " + sdk,
                "PATH": str(deps / "bin") + ":/usr/bin:/bin:/usr/sbin:/sbin", "PKG_CONFIG": "/usr/bin/false", "PKG_CONFIG_PATH": str(deps / "lib/pkgconfig"), "PKG_CONFIG_LIBDIR": str(deps / "lib/pkgconfig")}
    order = [PINS[2], PINS[3], PINS[4], PINS[5]]
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
        elif name in ("libgcrypt", "libassuan"):
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
    build_git(run, candidate, compiler, sdk, buildenv)
    source = run / "src/gnupg-2.5.24"
    work = private_dir(run / "build/gnupg")
    _, helpout, _ = command(run, [str(source / "configure"), "--help"], cwd=work, extra_env=buildenv)
    flags = ["--prefix=" + str(candidate), "--disable-gpgsm", "--disable-scdaemon", "--disable-dirmngr", "--disable-keyboxd", "--disable-tpm2d", "--disable-g13", "--disable-doc", "--disable-gpgtar", "--disable-wks-tools", "--disable-tests", "--disable-exec", "--disable-photo-viewers", "--disable-card-support", "--disable-ccid-driver", "--disable-dirmngr-auto-start", "--disable-sqlite", "--disable-tofu", "--disable-ldap", "--disable-gnutls", "--disable-ntbtls", "--disable-libdns", "--disable-bzip2", "--disable-nls", "--without-readline", "--with-libgpg-error-prefix=" + str(deps), "--with-libgcrypt-prefix=" + str(deps), "--with-libassuan-prefix=" + str(deps), "--with-npth-prefix=" + str(deps)]
    for flag in flags[1:]:
        require(supported_option(flag, helpout), f"unrecognized GnuPG configure option: {flag}")
    status, out, err = command(run, [str(source / "configure"), *flags], cwd=work, extra_env=buildenv, allow_failure=True)
    require(b"unrecognized option" not in out + err, "GnuPG configure ignored an option")
    if status:
        exclusive(run / "unsupported.json", canonical({"reason": "GnuPG configure failed; no dependency substitution authorized", "exit_code": status, "candidate": str(candidate), "build_id": run.name}))
        raise Refused(f"GnuPG configure exit={status}; inspect original logs and generated config in {run}; no additional source/library authorized")
    raise Refused("configure completed; guarded compilation pending source review")


def build_git(run, candidate, compiler, sdk, buildenv):
    # Git's release tarball is already generated, so no autotools or external
    # package discovery is used. Real supported read/signature paths remain.
    work = run / "src/git-2.54.0"
    cflags = "-O2 -g0 -isysroot " + sdk + " -DDREAM_PROOF_GPG='\"" + str(candidate / "bin/gpg") + "\"'"
    variables = ["CC=" + compiler, "CFLAGS=" + cflags, "LDFLAGS=-isysroot " + sdk, "NO_PTHREADS=YesPlease", "NO_CURL=YesPlease", "NO_EXPAT=YesPlease", "NO_OPENSSL=YesPlease", "NO_GETTEXT=YesPlease", "NO_TCLTK=YesPlease", "NO_PERL=YesPlease", "NO_PYTHON=YesPlease", "USE_LIBPCRE=", "NO_FINK=YesPlease", "NO_DARWIN_PORTS=YesPlease", "NO_APPLE_COMMON_CRYPTO=YesPlease", "prefix=" + str(candidate)]
    command(run, ["/usr/bin/make", "-j2", *variables, "git"], cwd=work, extra_env=buildenv)
    bindir = private_dir(candidate / "bin")
    exclusive(bindir / "git", read_regular(work / "git"), executable=True)
    command(run, ["/usr/bin/otool", "-L", str(bindir / "git")])
    command(run, ["/usr/bin/otool", "-l", str(bindir / "git")])
    for name, _, *_ in PINS:
        version = next(p[1] for p in PINS if p[0] == name)
        licensedir = private_dir(candidate / "licenses")
        part = private_dir(licensedir / name)
        for path in (run / "src" / f"{name}-{version}").glob("COPYING*"):
            exclusive(part / path.name, read_regular(path))
    # Version/image output is provenance only, not a lifetime qualification.
    command(run, [str(bindir / "git"), "--version"])


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
    require(len(data) + len(lock_data) + len(report) <= 131072, "TOTAL prepared evidence exceeds 128KiB")
    fields(q["tools"], "git gpg"); refs += list(q["tools"].values())
    checked_array(q["sources"]); require(len(q["sources"]) == 6, "six source authentications required")
    for row, source in zip(q["sources"], PINS):
        fields(row, "name archive_sha256 signature_sha256 signer_primary signer_subkey verifier_sha256 auth_exit auth_status_sha256")
        require(row["name"] == source[0] and row["archive_sha256"] == source[3] and row["signature_sha256"] == source[4] and row["verifier_sha256"] == VERIFIER_SHA and row["auth_exit"] == 0, "source authentication binding")
        for k in ("archive_sha256", "signature_sha256", "verifier_sha256", "auth_status_sha256"): hex_string(row[k])
        for k in ("signer_primary", "signer_subkey"): hex_string(row[k], 40, True)
        require(row["signer_primary"] in ([GIT_ROOT] if source[0] == "git" else GPG_ROOTS), "signer root")
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
            old = strict_json(read_regular(previous / "component-start.json", 32768, True), 32768)
            require(old != key, f"unknown original component effect retained; inspect {previous}; no retry")
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


def check(d, component_only=False):
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
            check(d, args.component_only)
    except (Refused, OSError, subprocess.TimeoutExpired) as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
