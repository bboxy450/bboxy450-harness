#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Check the steering family with the canonical pinned native/TLC comparator."""
import importlib.util
import json
import os
from pathlib import Path
import sys

sys.dont_write_bytecode = True
root = Path(__file__).resolve().parents[1]
target = Path(os.environ.get("TARGET_DIR", "target")).resolve()
model_root = target / "acp-steering/formal"
spec = importlib.util.spec_from_file_location("formal", root / "scripts/formal.py")
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
java, jar, _ = formal.tools(target / "formal")
expected_names = {"HarnessAcpSteering", "BrokenHarnessAcpSteeringReplay", "BrokenHarnessAcpSteeringCancel"}
checked = []
for line in (model_root / "manifest.tsv").read_text().splitlines():
    name, expected, count, width = line.split("\t")
    assert name in expected_names
    graph = formal.tlc(java, jar, model_root, name, True)
    assert formal.verdict(graph) == "pass", graph.stdout
    actual = formal.dot_graph(model_root / f"{name}.dot", int(width))
    assert actual == formal.native_graph(model_root, name)
    assert len(actual[0]) == int(count)
    verdict = formal.tlc(java, jar, model_root, name, False)
    assert formal.verdict(verdict) == expected, verdict.stdout
    checked.append(dict(name=name, verdict=expected, states=len(actual[0]), edges=len(actual[1])))
    print(f"{name}: {expected}; {len(actual[0])} states and {len(actual[1])} edges match", flush=True)
assert {row["name"] for row in checked} == expected_names
(model_root / "comparison.json").write_text(json.dumps(checked, indent=2) + "\n")
