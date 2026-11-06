#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused decision66050 parity; never a full repository formal receipt."""
import argparse
from concurrent.futures import ThreadPoolExecutor
import hashlib
import importlib.util
import json
from pathlib import Path
import re
import sys

sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[3]
SUFFIXES = (
    "", "ErrnoAck", "RetryStop", "Resume", "LeaderOnly", "Roster", "Identity",
    "FailedEndpoint", "Controller", "Pipe", "Late", "Quantum", "Clock", "Debt",
    "Reap", "UnknownExit", "WorkWhilePending", "PayloadWhilePending",
)
NAMES = [("Broken" if index else "") + "HarnessDreamReadFailedStop" + suffix
         for index, suffix in enumerate(SUFFIXES)]
EXPECTED_PROPERTIES = {
    "ErrnoAck": "ConsumedFailureNeverBecomesStopAck",
    **{name: "PendingTerminalNeverRepeatsStopOrContinuesOrWorks"
       for name in ("RetryStop", "Resume", "WorkWhilePending", "PayloadWhilePending")},
    **{name: "TerminalNeedsEveryIndependentOriginalFact" for name in (
        "LeaderOnly", "Roster", "Identity", "FailedEndpoint", "Controller",
        "Pipe", "Late", "Quantum", "Clock", "UnknownExit")},
    **{name: "ReleaseRequiresActualSettlementAndFullPaidDebt" for name in ("Debt", "Reap")},
}


def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--target", type=Path, required=True,
                        help="Focused export directory containing models/manifest.tsv")
    parser.add_argument("--tools-target", type=Path, default=ROOT / "target/formal")
    parser.add_argument("--model-source", type=Path, default=ROOT / "tests/formal/dream_read.rs",
                        help="Exact model source used by the focused native exporter")
    args = parser.parse_args()
    target = args.target.resolve()
    models = target / "models"
    spec = importlib.util.spec_from_file_location("canonical_formal", ROOT / "scripts/formal.py")
    formal = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(formal)
    inputs = [args.model_source.resolve(), *(ROOT / name for name in (
        "tests/formal/models.rs", "tests/formal/export.rs", "verification/contracts.toml",
        "verification/corpus.toml", "verification/coverage.toml", "verification/tools.lock.json",
        "scripts/formal.py", "tests/fixtures/dream-read/check-terminal-models.py"))]
    before = {str(path): digest(path) for path in inputs}
    source_paths = [ROOT / name for name in (
        "src/dream/c1.rs", "src/dream/read.rs", "src/dream/resolve.rs",
        "tests/fixtures/dream-read/proof-helper-probe.c")]
    source_before = {str(path): digest(path) for path in source_paths}
    java, jar, pins = formal.tools(args.tools_target.resolve())
    rows = [line.split("\t") for line in (models / "manifest.tsv").read_text().splitlines()]
    if [row[0] for row in rows] != NAMES:
        raise ValueError("focused native manifest must contain exactly all18 ordered decision66050 models")
    generated_inputs = [models / "manifest.tsv", *(models / (name + suffix)
        for name in NAMES for suffix in (
            ".tla", ".cfg", "Graph.cfg", ".metadata.json", ".states", ".edges", ".native-trace"))]
    generated_before = {str(path): digest(path) for path in generated_inputs}

    def check(item):
        index, row = item
        name, expected, count, width = row
        wanted = "safety" if index else "pass"
        record = {"name": name, "expected": wanted, "passed": False}
        try:
            if expected != wanted:
                raise ValueError("manifest expected verdict differs")
            metadata = formal.contract_metadata(models, name)
            native_names = set(re.findall(
                r"^(\w+) prefix=", (models / (name + ".native-trace")).read_text(), re.MULTILINE))
            exact_names = {EXPECTED_PROPERTIES[SUFFIXES[index]]} if index else set()
            if native_names != exact_names or "TypeOK" in native_names:
                raise ValueError("native causal named violation differs")
            native = formal.native_graph(models, name)
            graph = formal.tlc(java, jar, models, name, True)
            record["graph_exit"] = graph.returncode
            if formal.verdict(graph) != "pass":
                raise ValueError("reference graph exploration did not complete")
            reference = formal.dot_graph(models / (name + ".dot"), int(width))
            record.update(states=len(reference[0]), labelled_edges=len(reference[1]),
                          graph_equal=native == reference)
            if native != reference or len(reference[0]) != int(count):
                raise ValueError("complete reachable state/labelled edge disagreement")
            verdict = formal.tlc(java, jar, models, name, False)
            record["verdict_exit"] = verdict.returncode
            if formal.verdict(verdict) != wanted:
                raise ValueError("reference safety/deadlock/liveness verdict differs")
            symbols = {f"Inv{i}": value for i, value in enumerate(metadata["invariants"])}
            violated = re.findall(r"Invariant (\w+) is violated", verdict.stdout)
            if any(symbol not in symbols for symbol in violated):
                raise ValueError("reference TypeOK or unknown invariant is not a causal control")
            reference_names = {symbols[symbol] for symbol in violated}
            if reference_names != native_names:
                raise ValueError("reference/native complete named failure sets differ")
            record.update(native_named_verdicts=sorted(native_names),
                          reference_named_verdicts=sorted(reference_names), passed=True)
        except Exception as error:
            record["error"] = str(error)
        print(json.dumps(record, sort_keys=True), flush=True)
        return record

    with ThreadPoolExecutor(max_workers=2) as pool:
        results = list(pool.map(check, enumerate(rows)))
    after = {str(path): digest(path) for path in inputs}
    source_after = {str(path): digest(path) for path in source_paths}
    generated_after = {str(path): digest(path) for path in generated_inputs}
    unchanged = before == after and generated_before == generated_after
    passed = unchanged and all(row["passed"] for row in results)
    evidence = {
        "scope": "decision66050 all18 finite models only; no full-corpus or implementation refinement receipt",
        "passed": passed, "exit_code": 0 if passed else 1, "models": results,
        "native_state_cap": 100000, "reference_timeout_seconds": 60,
        "source_sha256_before": before, "source_sha256_after": after,
        "generated_model_sha256_before": generated_before,
        "generated_model_sha256_after": generated_after,
        "source_unchanged": unchanged, "implementation_sha256_before": source_before,
        "implementation_sha256_after": source_after,
        "implementation_source_unchanged": source_before == source_after,
        "implementation_checks_executed": False,
        "java_executable_sha256": digest(java), "tlc_jar_sha256": digest(jar),
        "tool_pins": pins,
    }
    (target / "terminal-parity-evidence.json").write_text(
        json.dumps(evidence, sort_keys=True, indent=2) + "\n")
    return 0 if passed else 1


if __name__ == "__main__":
    raise SystemExit(main())
