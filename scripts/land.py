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
import shutil
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


def gate(base):
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
        run(["cargo", "nextest", "run", "--offline", "--locked", "--jobs", "2", "-p", "bboxy450-harness", "--features", "acp-agent,test-fixtures", "--lib", "--target-dir", TARGET], env=env)
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
            integrations.remove("dream_read")
        if any(p.startswith(("src/mcp", "src/python_execute", "src/backend/acp_agent", "src/engine")) for p in changed):
            integrations.update(["mcp_client", "acp_launch_context"])
        if integrations:
            args = ["cargo", "nextest", "run", "--offline", "--locked", "--jobs", "2", "-p", "bboxy450-harness", "--features", "acp-agent,test-fixtures", "--target-dir", TARGET]
            for test in sorted(integrations - {"formal_models", "formal_nonvacuity", "reactor_projection"}):
                args.extend(["--test", test])
            if "--test" in args:
                run(args, env=env)
        enclosing = ROOT.parent.parent
        embedded = (enclosing / "scripts/formal.py").is_file()
        if embedded:
            env["BBOXY_TOOLKIT_ROOT"] = str(enclosing)
        # Do not depend on unlanded app/workspace changes, or overwrite their
        # aggregate evidence. Cargo uses the selected private target for formal artifacts.
        formal_target=TARGET/"harness-landing"
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


if __name__ == "__main__":
    try:
        main()
    except (RuntimeError, subprocess.SubprocessError) as error:
        print(f"land: {error}", file=sys.stderr)
        sys.exit(77)
