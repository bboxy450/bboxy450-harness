#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Signed lean landing, with source-bound local command/exit evidence.

Adapted from bboxy450-term scripts/land-fast.sh:
https://github.com/bboxy450/bboxy450-term/blob/main/scripts/land-fast.sh
This single-crate gate reuses Make checks and verifies its own formal corpus.
An enclosing app is landed separately with its combined current-source gate.
"""
import hashlib
import json
import os
import re
import shutil
import stat
from pathlib import Path
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("TARGET_DIR", ROOT / "target")).resolve()
ROWS = []


def git(*args):
    return subprocess.check_output(["git", *args], cwd=ROOT, text=True).strip()


def run(args, *, cwd=ROOT, env=None, required=True):
    print("land:", " ".join(map(str, args)), flush=True)
    result = subprocess.run(list(map(str, args)), cwd=cwd, env=env)
    ROWS.append({"argv": list(map(str, args)), "cwd": str(cwd), "exit": result.returncode})
    if required and result.returncode:
        raise RuntimeError(f"gate failed: {args[0]} exited {result.returncode}")
    return result.returncode


def source_digest():
    digest = hashlib.sha256()
    for name in git("ls-files", "-z").split("\0"):
        path = ROOT / name
        if name and path.is_file():
            digest.update(name.encode() + b"\0" + path.read_bytes() + b"\0")
    return digest.hexdigest()


def preserve_dream_observer_outputs():
    """Retain canonical sink-test evidence without treating it as source."""
    if (not ROWS or ROWS[-1].get("argv", [])[:2] != ["make", "test-dream-mcp"]
            or ROWS[-1].get("exit") != 0):
        raise RuntimeError("Dream observer preservation requires the successful canonical gate")
    if git("diff", "--name-only", "HEAD"):
        raise RuntimeError("tracked source changed during Dream gate")
    names = [name for name in git("ls-files", "--others", "--exclude-standard", "-z").split("\0") if name]
    # Run evidence is machine-local (.bboxy/.gitignore); find the observer outputs there.
    names += sorted(str(p.relative_to(ROOT)) for p in (ROOT / ".bboxy/runs").glob("dream-wp8-observer-*.json"))
    expected = {"good", "aba", "wrong_temp", "omit_dir_sync", "hand_race"}
    pattern = re.compile(r"\.bboxy/runs/dream-wp8-observer-(good|aba|wrong_temp|omit_dir_sync|hand_race)-[0-7][0-9A-HJKMNP-TV-Z]{25}\.json")
    records = []
    for name in names:
        match = pattern.fullmatch(name)
        if not match or match[1] not in expected:
            raise RuntimeError(f"unexpected untracked output during Dream gate: {name}")
        expected.remove(match[1])
        path = ROOT / name
        with open(path, "rb", opener=lambda p, flags: os.open(p, flags | os.O_NOFOLLOW)) as stream:
            identity = os.fstat(stream.fileno())
            if (not stat.S_ISREG(identity.st_mode) or identity.st_nlink != 1
                    or identity.st_uid != os.geteuid()):
                raise RuntimeError(f"Dream observer output is not an owned regular file: {name}")
            raw = stream.read(65537)
            current = os.lstat(path)
            fields = ("st_dev", "st_ino", "st_mode", "st_uid", "st_nlink", "st_size", "st_mtime_ns", "st_ctime_ns")
            if len(raw) > 65536 or any(getattr(current, key) != getattr(identity, key) for key in fields):
                raise RuntimeError(f"Dream observer output changed or exceeded its bound: {name}")
        value = json.loads(raw)
        if (value.get("fault") != match[1]
                or value.get("accepted_by_independent_observer") is not (match[1] in {"good", "hand_race"})):
            raise RuntimeError(f"Dream observer output differs from its completed case: {name}")
        records.append({"source": name, "file": path.name, "device": identity.st_dev,
                        "inode": identity.st_ino, "bytes": len(raw), "sha256": hashlib.sha256(raw).hexdigest()})
    if expected:
        raise RuntimeError("canonical Dream gate did not produce every observer case")
    destination = Path(tempfile.mkdtemp(prefix="dream-observer-evidence-", dir=TARGET))
    manifest = destination / "preservation.json"
    manifest.write_text(json.dumps({"method": "same-filesystem rename; retain original inodes and bytes",
                                    "source_commit": git("rev-parse", "HEAD"), "gate": ROWS[-1],
                                    "files": records}, indent=2) + "\n")
    for record in records:
        source, retained = ROOT / record["source"], destination / record["file"]
        original = os.lstat(source)
        if (original.st_dev, original.st_ino, original.st_size, original.st_nlink) != (
                record["device"], record["inode"], record["bytes"], 1):
            raise RuntimeError("Dream observer output changed before preservation")
        source.rename(retained)
        actual = os.lstat(retained)
        with open(retained, "rb", opener=lambda p, flags: os.open(p, flags | os.O_NOFOLLOW)) as stream:
            retained_raw = stream.read(65537)
        if ((actual.st_dev, actual.st_ino, actual.st_size) != (
                record["device"], record["inode"], record["bytes"])
                or hashlib.sha256(retained_raw).hexdigest() != record["sha256"]):
            raise RuntimeError("Dream observer preservation changed its identity or bytes")
    ROWS.append({"operation": "preserve-dream-observer-evidence", "manifest": str(manifest),
                 "manifest_sha256": hashlib.sha256(manifest.read_bytes()).hexdigest(), "exit": 0})
    print(f"land: retained five Dream observer outputs at {manifest}", flush=True)


# Serialized timing lane. These lib-test families measure real processes and
# millisecond deadlines; the canonical Dream targets run them optimized and one
# at a time (Makefile test-dream-c1 and test-dream-mcp: `--release ... --lib
# <family> -- --test-threads=1`). The concurrent debug nextest pass excludes
# them and land runs each one here the same way. Mirror the Makefile exactly;
# every entry is a full lib-test path prefix ending in `::`.
SERIALIZED_LIB_FAMILIES = (
    "dream::bootstrap::",                                  # Makefile test-dream-c1
    "dream::c1::",                                         # Makefile test-dream-c1
    "dream::fingerprint::",                                # Makefile test-dream-c1
    "frontends::headless::dream_c1_fixture_environment_tests::",  # test-dream-c1
    "mcp::source_tests::",                                 # Makefile test-dream-c1
    "dream::read_wire::",                                  # Makefile test-dream-mcp
    "dream::worker::tests::",                              # Makefile test-dream-mcp
    "dream::resolve::prepared_qualification_tests::",      # Makefile test-dream-mcp
    "dream::resolve::selected_helper_process_tests::",     # test-dream-mcp; flaked in land
    "dream::read::helper_tests::",                         # test-dream-mcp; flaked in land
)
LAND_LIB_FEATURES = "acp-agent,test-fixtures"


def concurrent_lib_filter(families=SERIALIZED_LIB_FAMILIES):
    """Nextest filterset that leaves every serialized family out of the concurrent pass."""
    return "not (" + " | ".join(f"test(/^{re.escape(f)}/)" for f in families) + ")"


def serialized_lib_commands(families=SERIALIZED_LIB_FAMILIES):
    return [["cargo", "test", "--offline", "--locked", "--release", "--jobs", "2", "-p", "bboxy450-harness",
             "--features", LAND_LIB_FEATURES, "--target-dir", TARGET, "--lib", family, "--", "--test-threads=1"]
            for family in families]


def check_serialized_lane(excluded_filter, commands):
    """A family left out of the concurrent pass must be run in the serialized lane, and vice versa."""
    excluded = re.findall(r"test\(/\^(.*?)/\)", excluded_filter)
    excluded = [re.sub(r"\\(.)", r"\1", f) for f in excluded]
    serialized = [c[c.index("--lib") + 1] for c in commands]
    if not excluded or len(set(excluded)) != len(excluded):
        raise RuntimeError(f"serialized timing filter is empty or repeats a family: {excluded}")
    missing = sorted(set(excluded) - set(serialized))
    if missing:
        raise RuntimeError(f"excluded from the concurrent pass but never run serialized: {missing}")
    extra = sorted(set(serialized) - set(excluded))
    if extra:
        raise RuntimeError(f"run serialized but not excluded from the concurrent pass: {extra}")
    for command in commands:
        if command[-2:] != ["--", "--test-threads=1"] or "--release" not in command:
            raise RuntimeError(f"serialized timing command is not release/--test-threads=1: {command}")
    for family in serialized:
        if not family.endswith("::"):
            raise RuntimeError(f"serialized family is not a full path prefix: {family}")


def run_serialized_family(args, env):
    """Run one family; a family that matches zero tests was excluded yet never run."""
    print("land:", " ".join(map(str, args)), flush=True)
    result = subprocess.run(list(map(str, args)), cwd=ROOT, env=env, stdout=subprocess.PIPE, text=True)
    sys.stdout.write(result.stdout)
    sys.stdout.flush()
    counts = [int(n) for n in re.findall(r"^running (\d+) tests?$", result.stdout, re.M)]
    ROWS.append({"argv": list(map(str, args)), "cwd": str(ROOT), "exit": result.returncode, "tests": sum(counts)})
    if result.returncode:
        raise RuntimeError(f"gate failed: serialized timing family {args[args.index('--lib') + 1]} exited {result.returncode}")
    if sum(counts) == 0:
        raise RuntimeError(f"serialized timing family {args[args.index('--lib') + 1]} ran zero tests")


def test_features(name):
    """Features an integration-test file gates itself on with `#![cfg(feature = "...")]`."""
    head = (ROOT / "tests" / f"{name}.rs").read_text().split("\nfn ", 1)[0]
    gates = re.findall(r"^#!\[cfg\((.*)\)\]", head, re.M)
    return tuple(sorted({f for gate in gates for f in re.findall(r'feature\s*=\s*"([^"]+)"', gate)}))


def self_test():
    """The gate must compile GUI tests with `embedded`, or they report zero tests."""
    for name in ("gui_rich", "gui_render", "gui_card_bounds"):
        assert test_features(name) == ("embedded",), (name, test_features(name))
    assert test_features("mcp_client") == ("test-fixtures",), test_features("mcp_client")
    # The two lane lists agree, and a family excluded but not run is an error.
    check_serialized_lane(concurrent_lib_filter(), serialized_lib_commands())
    for family in ("dream::resolve::selected_helper_process_tests::", "dream::read::helper_tests::"):
        assert family in SERIALIZED_LIB_FAMILIES, family
    dropped = serialized_lib_commands(SERIALIZED_LIB_FAMILIES[:-1])
    try:
        check_serialized_lane(concurrent_lib_filter(), dropped)
    except RuntimeError as error:
        assert "never run serialized" in str(error), error
    else:
        raise AssertionError("an excluded family without a serialized run must fail the gate")
    try:
        check_serialized_lane(concurrent_lib_filter(SERIALIZED_LIB_FAMILIES[1:]), serialized_lib_commands())
    except RuntimeError as error:
        assert "not excluded" in str(error), error
    else:
        raise AssertionError("a serialized family still in the concurrent pass must fail the gate")
    # Every Makefile `--release ... --lib <family> -- --test-threads=1` is mirrored.
    makefile = (ROOT / "Makefile").read_text()
    canonical = set(re.findall(r"--release[^\n]*?--lib (\S+) -- --test-threads=1", makefile))
    mirrored = {f for f in SERIALIZED_LIB_FAMILIES}
    unmatched = [c for c in canonical if not any(c in f for f in mirrored)]
    assert not unmatched, f"Makefile serializes lib families land does not: {unmatched}"
    assert all(any(c in f for c in canonical) for f in mirrored), (canonical, mirrored)


def gate(base):
    self_test()
    changed = git("diff", "--name-only", f"{base}...HEAD").splitlines()
    # Native controller deadlines include process birth/Hello. Keep their
    # canonical ordinary scheduling precondition; background only rustc below.
    priority = os.getpriority(os.PRIO_PROCESS, 0)
    if priority != 0:
        raise RuntimeError(f"start make land at ordinary priority (nice=0), got {priority}")
    compiler_wrapper = shutil.which("nice")
    if not compiler_wrapper:
        raise RuntimeError("nice is required to keep compiler work in the background")
    print(f"land: controller nice={priority}; compiler wrapper={compiler_wrapper} (default increment 10)", flush=True)
    env = {k: v for k, v in os.environ.items() if not k.startswith("BBOXY_")}
    env.update(CARGO_HOME=os.environ.get("CARGO_HOME", str(Path.home() / ".cargo")),
               RUSTUP_HOME=os.environ.get("RUSTUP_HOME", str(Path.home() / ".rustup")),
               CARGO_BUILD_JOBS="2", BBOXY_CARGO_JOBS="2", BBOXY_CACHE_MODE="off",
               NEXTEST_TEST_THREADS="2", RUST_TEST_THREADS="2",
               BBOXY_CARGO_TARGET_DIR=str(TARGET), CARGO_TARGET_DIR=str(TARGET),
               CARGO_INCREMENTAL="0",CARGO_BUILD_BUILD_DIR=str(TARGET),
               # Cargo passes the actual rustc executable as argv[1]. POSIX nice
               # defaults to +10; Cargo/test controllers remain unwrapped.
               RUSTC_WRAPPER=compiler_wrapper,RUSTC_WORKSPACE_WRAPPER="")
    run([compiler_wrapper, sys.executable, "-I", "-c",
         "import os; n=os.getpriority(os.PRIO_PROCESS,0); print('land: compiler-wrapper nice='+str(n),flush=True); assert n==10"], env=env)
    with tempfile.TemporaryDirectory(prefix="harness-land-") as private:
        env["HOME"] = private
        env["GNUPGHOME"] = str(Path(private) / "gnupg")
        Path(env["GNUPGHOME"]).mkdir(mode=0o700)
        # Only public verification material enters the private test home.
        fingerprints = sorted(set(git("log", "--format=%GF", "origin/main..HEAD").splitlines()))
        public = subprocess.check_output(["gpg", "--batch", "--export", *fingerprints])
        imported = subprocess.run(["gpg", "--homedir", env["GNUPGHOME"], "--batch", "--import"], input=public)
        ROWS.append({"argv":["gpg","--batch","--import","<public verification keys>"],"exit":imported.returncode})
        if imported.returncode:
            raise RuntimeError("cannot prepare public signature verification material")
        run(["git", "diff", "--check", f"{base}...HEAD"], env=env)
        run(["make", "test-licenses", f"TARGET_DIR={TARGET}"], env=env)
        run(["make", "check-embedded-lints", f"TARGET_DIR={TARGET}"], env=env)
        run(["cargo", "clippy", "--offline", "--locked", "--jobs", "2", "-p", "bboxy450-harness", "--all-targets", "--features", "acp-agent", "--target-dir", TARGET, "--", "-D", "warnings"], env=env)
        run(["cargo", "clippy", "--offline", "--locked", "--jobs", "2", "-p", "bboxy450-harness", "--features", "acp-agent,test-fixtures", "--lib", "--test", "mcp_client", "--test", "acp_launch_context", "--test", "formal_models", "--target-dir", TARGET, "--", "-D", "warnings"], env=env)
        lib_filter, serialized = concurrent_lib_filter(), serialized_lib_commands()
        check_serialized_lane(lib_filter, serialized)
        run(["cargo", "nextest", "run", "--offline", "--locked", "--jobs", "2", "-p", "bboxy450-harness", "--features", LAND_LIB_FEATURES, "--lib", "--target-dir", TARGET, "-E", lib_filter], env=env)
        for command in serialized:
            run_serialized_family(command, env)
        integrations = {Path(p).stem for p in changed if p.startswith("tests/") and p.count("/") == 1 and p.endswith(".rs")}
        if "dream_c1" in integrations:
            # These native process fixtures require the explicitly prepared
            # ordinary release worker, permit corpus and broken-control worker.
            # Running their executable in an unprepared generic nextest lane is
            # not a valid test environment; keep the existing canonical gate.
            run(["make", "test-dream-c1-integration", f"TARGET_DIR={TARGET}"], env=env)
            integrations.remove("dream_c1")
        if "dream_read" in integrations:
            # Q2 requires the canonical source-bound release worker/profile and
            # its real proof/resource gates, prepared by this existing target.
            run(["make", "test-dream-mcp", f"TARGET_DIR={TARGET}"], env=env)
            preserve_dream_observer_outputs()
            integrations.remove("dream_read")
        if any(p.startswith(("src/mcp", "src/python_execute", "src/backend/acp_agent", "src/engine")) for p in changed):
            integrations.update(["mcp_client", "acp_launch_context"])
        # Each test file runs with the features its crate-level cfg requires; a
        # binary that still reports zero tests remains nextest's error (exit 4).
        groups = {}
        for test in sorted(integrations - {"formal_models", "formal_nonvacuity", "reactor_projection"}):
            groups.setdefault(test_features(test), []).append(test)
        for features, tests in sorted(groups.items()):
            args = ["cargo", "nextest", "run", "--offline", "--locked", "--jobs", "2", "-p", "bboxy450-harness", "--features", ",".join(sorted({"acp-agent", "test-fixtures", *features})), "--target-dir", TARGET]
            for test in tests:
                args.extend(["--test", test])
            run(args, env=env)
        enclosing = ROOT.parent.parent
        embedded = (enclosing / "scripts/formal.py").is_file()
        if embedded:
            env["BBOXY_TOOLKIT_ROOT"] = str(enclosing)
        # Do not depend on unlanded app/workspace changes, or overwrite their
        # aggregate evidence. Cargo uses the selected private target for formal artifacts.
        # Formal and runtime gates share the explicitly configured build tree.
        formal_target=TARGET
        if run(["make", "check-formal-evidence", f"TARGET_DIR={formal_target}"], env=env, required=False):
            run(["make", "test-formal-parity", "FORMAL_REUSE=1", f"TARGET_DIR={formal_target}"], env=env)
        run(["make", "check-formal-evidence", f"TARGET_DIR={formal_target}"], env=env)


def main():
    os.environ["GIT_TERMINAL_PROMPT"] = "0"
    if git("status", "--porcelain"):
        raise RuntimeError("commit exact source paths before make land")
    TARGET.mkdir(parents=True, exist_ok=True)
    receipt = {"commands": ROWS}
    try:
        for _ in range(5):
            run(["git", "fetch", "origin", "main"])
            if run(["git", "rebase", "--gpg-sign", "origin/main"], required=False):
                raise RuntimeError("rebase conflict; resolve in this worktree and rerun make land")
            revisions = git("rev-list", "origin/main..HEAD").splitlines()
            if not revisions:
                print("land: nothing to land")
                return
            for revision in revisions:
                run(["git", "verify-commit", revision])
            receipt.update(head=git("rev-parse", "HEAD"), source_sha256=source_digest())
            gate("origin/main")
            if source_digest() != receipt["source_sha256"] or git("status", "--porcelain"):
                raise RuntimeError("source changed during verification")
            if not run(["git", "push", "origin", "HEAD:main"], required=False):
                receipt["landed"] = receipt["head"]
                print("LANDED", receipt["landed"])
                return
        raise RuntimeError("main kept moving; rerun make land")
    finally:
        path = TARGET / "harness-land-receipt.json"
        temporary = path.with_suffix(".tmp")
        temporary.write_text(json.dumps(receipt, indent=2) + "\n")
        temporary.replace(path)


if __name__ == "__main__" and sys.argv[1:] == ["--self-test"]:
    self_test()
    print("land: self-test ok")
elif __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.SubprocessError) as error:
        print(f"land: {error}", file=sys.stderr)
        sys.exit(77)
