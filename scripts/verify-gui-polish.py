#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused GUI changes only; reuse the canonical formal graph/verdict helpers."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys

ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get("TARGET_DIR", "target")).resolve()
OUT = TARGET / "gui-polish"
OUT.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, CARGO_BUILD_JOBS="2", RUST_TEST_THREADS="2",
           GUI_POLISH_FORMAL_OUT=str(OUT / "formal"), GUI_POLISH_CAPTURE_DIR=str(OUT / "captures"))
records = []
cargo = ["cargo", "--offline", "--locked", "--jobs", "2", "--target-dir", str(TARGET)]


def run(name, command):
    log = OUT / (name + ".log")
    with log.open("w") as stream:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    records.append({"command": command, "exit_code": result.returncode,
                    "log": str(log), "sha256": hashlib.sha256(log.read_bytes()).hexdigest()})
    print(f"gui-polish: {name}: exit {result.returncode}", flush=True)
    if result.returncode:
        print(log.read_text()[-6000:], file=sys.stderr)
        raise SystemExit(result.returncode)


files = ["ATTRIBUTION.md", "Makefile", "Cargo.toml", "Cargo.lock", "verification/tools.lock.json",
         "scripts/formal.py", "scripts/verify-gui-polish.py", "verification/coverage.toml",
         "verification/contracts.toml", "verification/corpus.toml", "tests/formal/models.rs",
         "tests/formal_models.rs", "tests/acp_agent.rs", "tests/gui_bridge.rs", "tests/gui_render.rs",
         "tests/gui_rich.rs", "tests/fixtures/acp-agent/fake_agent.py", "src/backend/acp_agent.rs",
         "src/frontends/face_core.rs"]
files += [str(p.relative_to(ROOT)) for p in (ROOT / "src/frontends/gui").rglob("*.rs")]
initial_hashes = {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in sorted(files)}

run("native", [cargo[0], "test", *cargo[1:], "--features", "embedded", "--lib",
               "--test", "gui_render", "--test", "gui_bridge", "--test", "gui_rich",
               "--test", "acp_agent", "--test", "formal_models", "gui_polish", "--", "--nocapture"])
run("embedded", [cargo[0], "check", *cargo[1:], "--no-default-features", "--features", "embedded", "--lib"])
run("standalone", [cargo[0], "check", *cargo[1:], "--features", "acp,standalone", "--lib", "--bin", "bboxy450-harness"])

spec = importlib.util.spec_from_file_location("formal", ROOT / "scripts/formal.py")
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
java, jar, pins = formal.tools(TARGET / "formal")
model_root = OUT / "formal"
models = []
for line in (model_root / "manifest.tsv").read_text().splitlines():
    name, expected, count, width = line.split("\t")
    if name not in {"HarnessGuiPendingPrompt", "BrokenHarnessGuiPendingPromptAck", "BrokenHarnessGuiPendingPromptStale"}:
        raise ValueError("unexpected model in focused manifest")
    graph = formal.tlc(java, jar, model_root, name, True)
    assert formal.verdict(graph) == "pass", name
    actual = formal.dot_graph(model_root / f"{name}.dot", int(width))
    native = formal.native_graph(model_root, name)
    assert actual == native and len(actual[0]) == int(count), f"graph disagreement: {name}"
    checked = formal.tlc(java, jar, model_root, name, False)
    assert formal.verdict(checked) == expected, f"verdict disagreement: {name}"
    models.append({"name": name, "expected": expected, "states": len(actual[0]),
                   "edges": len(actual[1]), "graph_equal": True, "graph_exit": graph.returncode,
                   "tlc_exit": checked.returncode})
    print(f"gui-polish: {name}: {expected}; {len(actual[0])} states, {len(actual[1])} transitions match", flush=True)
assert len(models) == 3
assert initial_hashes == {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in sorted(files)}, "source changed while checks ran"
evidence = {"verify": {"command": "make verify-gui-polish", "exit_code": 0},
            "source_head_at_check": subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=ROOT, text=True).strip(),
            "source_hashes": {p: hashlib.sha256((ROOT / p).read_bytes()).hexdigest() for p in sorted(files)},
            "commands": records, "models": models, "pins": pins, "operator_input": False,
            "coverage": "Named GUI-polish tests, changed finite contract/control native-TLC full graph and verdict parity, embedded and standalone compilation only.",
            "omitted": ["full suites", "formal corpus", "paid/live provider requests", "OS input", "install", "push"],
            "complete_tlc_replacement": False, "rust_refinement_proven": False}
(ROOT / ".bboxy/runs/codex-gui-polish.json").write_text(json.dumps(evidence, indent=2) + "\n")
