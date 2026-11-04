#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused pure parser, shared highlighting, native geometry/raster and embedded lint gate."""
from datetime import datetime, timezone
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("TARGET_DIR", ROOT / "target")).resolve()
assert ROOT in TARGET.parents, "Use this worktree's local target"
OUT = TARGET / "quiet-code-cards"
OUT.mkdir(parents=True, exist_ok=True)
CREATED = datetime.now(timezone.utc).isoformat()


def sha(data):
    return hashlib.sha256(data).hexdigest()


files = sorted(set(
    [str(p.relative_to(ROOT)) for p in (ROOT / "src").rglob("*.rs")]
    + [str(p.relative_to(ROOT)) for p in (ROOT / "tests/support").rglob("*.rs")]
    + ["tests/gui_rich.rs", "tests/fixtures/protocol/all-items.jsonl", "Cargo.toml",
       "Cargo.lock", "Makefile", "ATTRIBUTION.md", "scripts/verify-quiet-code-cards.py"]
))
hashes = {p: sha((ROOT / p).read_bytes()) for p in files}
env = dict(os.environ, RUSTC_WRAPPER="", CARGO_BUILD_JOBS="2", RUST_TEST_THREADS="2",
           CLIPPY_CONF_DIR=str(ROOT / "src/frontends/gui/lint"),
           QUIET_CODE_CAPTURE_DIR=str(OUT / "captures"))
commands = []


def run(name, verb, *args):
    command = ["cargo", verb, "--offline", "--locked", "--jobs", "2", "--target-dir",
               str(TARGET), "--no-default-features", "--features", "embedded", *args]
    log = OUT / (name + ".log")
    with log.open("w") as stream:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream,
                                stderr=subprocess.STDOUT)
    commands.append(dict(name=name, command=command, exit_code=result.returncode,
                         log=str(log.relative_to(ROOT)), sha256=sha(log.read_bytes())))
    print(f"quiet-code-cards: {name}: exit {result.returncode}", flush=True)
    if result.returncode:
        print(log.read_text()[-6000:], file=sys.stderr)
        raise SystemExit(result.returncode)


run("parser-source-and-shared-highlight", "test", "--lib", "quiet_code", "--", "--nocapture")
run("native-name-title-geometry-and-mcp-raster", "test", "--test", "gui_rich", "quiet_code", "--", "--nocapture")
run("strict-embedded-compile", "clippy", "--lib", "--no-deps", "--", "-D", "warnings")
assert hashes == {p: sha((ROOT / p).read_bytes()) for p in files}, "Inputs changed during verification"
captures = [dict(path=str(p.relative_to(ROOT)), sha256=sha(p.read_bytes()))
            for p in sorted((OUT / "captures").glob("*.png"))]
assert len(captures) == 2, "Both actual native MCP raster sizes must be captured"
receipt = dict(
    created=CREATED, verify=dict(command="make verify-quiet-code-cards", exit_code=0),
    source_head_at_check=subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
    commands=commands, source_hashes=hashes, captures=captures,
    scope="3 pure/parser/highlight units; 3 native name/title, 280/1120px, 90/100/160% geometry and >100KB MCP raster/scroll witnesses; strict embedded library lint.",
    dependency="Cargo.lock and Cargo.toml retain the existing bbt-doc/bbt-render 775527db16ffe33ee5cafaa0ff8566e3ea7512da pin; no dependency changes.",
    operator_input=False,
    omitted=["old ACP/formal gates", "full suites", "live vendors", "UI or backend restart",
             "desktop input", "dependency raster fix", "attachments", "integration", "install"])
(ROOT / ".bboxy/runs/codex-quiet-code-cards.json").write_text(json.dumps(receipt, indent=2) + "\n")
