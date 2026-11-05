#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused draft/spill-family native/reference parity using the canonical comparator."""
import importlib.util
import json
import os
from pathlib import Path
import sys
sys.dont_write_bytecode = True
root = Path(__file__).resolve().parents[1]
target = Path(os.environ.get("TARGET_DIR", "target")).resolve()
models = target / "pane/formal"
spec = importlib.util.spec_from_file_location("formal", root / "scripts/formal.py")
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
java, jar, _ = formal.tools(target / "formal")
expected_names = {"HarnessUnrestrictedStartup", "BrokenHarnessUnrestrictedStartup", "HarnessDraftHandoff", "BrokenHarnessDraftHandoff", "HarnessNativeSpill", "BrokenHarnessNativeSpill"}
rows = []
for line in (models / "manifest.tsv").read_text().splitlines():
    name, expected, count, width = line.split("\t")
    assert name in expected_names
    graph = formal.tlc(java, jar, models, name, True)
    assert formal.verdict(graph) == "pass", graph.stdout
    actual = formal.dot_graph(models / f"{name}.dot", int(width))
    assert actual == formal.native_graph(models, name) and len(actual[0]) == int(count)
    checked = formal.tlc(java, jar, models, name, False)
    assert formal.verdict(checked) == expected, checked.stdout
    rows.append(dict(name=name, verdict=expected, states=len(actual[0]), edges=len(actual[1])))
    print(f"{name}: {expected}; {len(actual[0])} states and {len(actual[1])} edges match", flush=True)
assert {row["name"] for row in rows} == expected_names
(models / "comparison.json").write_text(json.dumps(rows, indent=2) + "\n")
