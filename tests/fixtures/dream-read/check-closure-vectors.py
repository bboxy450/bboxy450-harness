#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Exercise the signed, synthetic-only C boundary corpus without native candidates.

The supplied binary is an offline parser wrapper: target/schema_ref/prefix argv,
exact wire on stdin, exit0 for acceptance and81 for refusal. It must have no
candidate, source discovery, helper birth, or publication path. Python checks
the separately frozen synthetic provenance, framing, observation and byte facts.
These tests make no claim about real linker-map parsing or native qualification.
"""

import argparse
from collections import Counter
import hashlib
import json
from pathlib import Path
import re
import subprocess
import sys


SCHEMA_SHA = "d70948ec375dae099ea1dac0b92dfb30ff87d7fcac94c98f42caa1c0ba2da8ec"
VECTORS_SHA = "bff32c1afb483980952a6761918d4005c9f1e12b66f3d7aaaa88f1b9afe09740"
U64_MAX = (1 << 64) - 1
TARGETS = {"closure", "schema_fragment", "canonical_grammar", "member_framing", "byte_accounting"}
CONTEXTUAL_INVARIANTS = {
    "frozen_image_binding", "frozen_map_binding", "platform_equality",
    "cache_boundary", "platform_observed_change", "stale_incarnation",
    "partial_publication",
}
LIMITATIONS = [
    "Synthetic codec evidence only; no real Basis, source authentication, candidate or native execution.",
    "Synthetic map blobs are opaque; exact map Ref and frozen selection_views are checked, not real linker syntax.",
    "Archive indexes use this corpus's LF-terminated synthetic name table; no real archive parser is qualified.",
    "Observations and buffer copies are supplied finite witnesses, not OS lifetime or arbitrary native heap proof.",
]


class HarnessError(Exception):
    """Bad corpus, unavailable parser, or a parser contract violation."""


class Refusal(Exception):
    """A semantic refusal, separate from a harness failure."""


def need(condition, detail):
    if not condition:
        raise Refusal(detail)


def digest(wire):
    return hashlib.sha256(wire).hexdigest()


def unique_object(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError("duplicate JSON key")
        result[key] = value
    return result


def invalid_number(_):
    raise ValueError("noninteger JSON number")


def load_json(wire):
    return json.loads(wire.decode("utf-8"), object_pairs_hook=unique_object,
                      parse_float=invalid_number, parse_constant=invalid_number)


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def unhex(text):
    if not isinstance(text, str) or len(text) % 2 or re.fullmatch(r"[0-9a-f]*", text) is None:
        raise HarnessError("noncanonical hex in signed corpus")
    return bytes.fromhex(text)


def unsigned(value, maximum=U64_MAX):
    return type(value) is int and 0 <= value <= maximum


def pinned_json(path, expected, length):
    wire = path.read_bytes()
    if len(wire) != length or digest(wire) != expected:
        raise HarnessError(f"signed artifact bytes differ: {path}")
    return load_json(wire)


def framed_members(wire):
    members = []
    offset = 0
    while offset < len(wire):
        need(len(wire) - offset >= 4, "truncated member length")
        size = int.from_bytes(wire[offset:offset + 4], "big")
        offset += 4
        need(1 <= size <= 4096 and size <= len(wire) - offset, "member extent")
        name = wire[offset:offset + size]
        need(not any(c in name for c in (b"\0", b"\r", b"\n")), "member name encoding")
        members.append(name)
        offset += size
    need(members, "empty member selection")
    return members


def check_ref(ref, artifacts):
    need(set(ref) == {"len", "path", "sha256"}, "offline Ref shape")
    need(ref["path"] in artifacts, "offline Ref outside finite artifact context")
    wire = artifacts[ref["path"]]
    need(unsigned(ref["len"]) and ref["len"] == len(wire)
         and ref["sha256"] == digest(wire), "offline Ref payload binding")


def validate_context(context):
    """Validate each immutable synthetic context independently of vector claims."""
    need(context["kind"] == "synthetic_codec_context_only", "context scope")
    artifacts = {}
    for artifact in context["artifacts"]:
        need(set(artifact) == {"path", "wire_hex"}, "artifact shape")
        need(artifact["path"] not in artifacts, "duplicate context artifact")
        artifacts[artifact["path"]] = unhex(artifact["wire_hex"])
    for image in context["image_views"]:
        check_ref(image["file"], artifacts)
    archives = {}
    indexes = {}
    for archive in context["archive_views"]:
        need(archive["id"] not in archives, "duplicate context archive")
        archives[archive["id"]] = archive
        check_ref(archive["file"], artifacts)
        check_ref(archive["member_index"], artifacts)
        index = artifacts[archive["member_index"]["path"]]
        need(index.endswith(b"\n"), "synthetic index is not LF terminated")
        names = index[:-1].split(b"\n")
        need(all(1 <= len(n) <= 4096 and b"\0" not in n and b"\r" not in n for n in names),
             "synthetic index has ambiguous names")
        need(len(set(names)) == len(names), "ambiguous synthetic index duplicate")
        indexes[archive["id"]] = set(names)
    selections = {}
    selected_keys = set()
    for selected in context["selection_views"]:
        need(set(selected) == {"archive", "member_frame_hex", "output", "tool"}, "selection shape")
        key = (selected["tool"], selected["output"])
        archive_id = selected["archive"]
        need(archive_id in archives and (*key, archive_id) not in selected_keys,
             "unknown or duplicate selected archive")
        selected_keys.add((*key, archive_id))
        wire = unhex(selected["member_frame_hex"])
        members = framed_members(wire)
        need(all(member in indexes[archive_id] for member in members), "selected member absent from archive index")
        need(len(members) <= (1 << 32) - 1, "selected occurrence count overflow")
        selections.setdefault(key, []).append({"id": archive_id,
                                              "member_count": len(members),
                                              "members_sha256": digest(wire)})
    maps = set()
    for link_map in context["map_views"]:
        check_ref(link_map["file"], artifacts)
        key = (link_map["tool"], link_map["output"])
        need(key not in maps, "duplicate context output map")
        maps.add(key)
        need(link_map["statics"] == selections.get(key, []), "offline selected count/hash/order differs")
    need(set(selections) <= maps, "selection has no corresponding output map")


def closure_context(closure, context, vector):
    need(closure["platform"] == context["build_platform"], "Build/Closure Platform6 differs")
    need(closure["images"] == context["image_views"], "frozen image/source/file/Mach-O differs")
    need(closure["statics"] == context["archive_views"], "frozen archive/index/source differs")
    need(closure["link_maps"] == context["map_views"], "frozen output map/selection differs")
    for image in closure["images"]:
        for load in image["macho"]["loads"]:
            if load["target"]["kind"] == "cache":
                need(load["name"] in context["cache_names"], "image absent from bound offline cache")
    need(vector["publication"] == "full", "partial publication")
    observations = vector["observations"]
    if observations is not None:
        need(set(observations) == {"captured_incarnation", "expected_incarnation", "platforms"}, "observation shape")
        captured, expected = observations["captured_incarnation"], observations["expected_incarnation"]
        need(unsigned(captured) and captured > 0 and captured == expected, "stale original incarnation")
        need(len(observations["platforms"]) >= 2, "before/after observations missing")
        # Every observation is checked. An intermediate mismatch stays failed,
        # including A -> B -> A; equality of only the endpoints is insufficient.
        need(all(p == closure["platform"] for p in observations["platforms"]), "observed Platform6 change")


def check_budget(budget, limits):
    need(isinstance(budget, dict) and set(budget) == {"buffers", "reserved_physical_bytes"}, "budget shape")
    caps = {"basis": limits["basis_bytes"], "descriptor": limits["descriptor_bytes"],
            "source_lock": limits["source_lock_bytes"], "result": limits["result_bytes_including_lf"],
            "receipt": limits["receipt_bytes_separate"], "qualification": limits["aggregate_bytes"]}
    logical = physical = receipt = 0
    per_kind = {kind: 0 for kind in caps}
    for buffer in budget["buffers"]:
        need(set(buffer) == {"copies", "kind", "wire_hex"} and buffer["kind"] in caps, "buffer shape/kind")
        size = len(unhex(buffer["wire_hex"]))
        copies = buffer["copies"]
        need(unsigned(copies) and copies > 0 and size <= caps[buffer["kind"]], "individual buffer limit")
        need(size * copies <= U64_MAX - physical, "physical copy arithmetic overflow")
        physical += size * copies
        per_kind[buffer["kind"]] += size
        need(per_kind[buffer["kind"]] <= caps[buffer["kind"]], "combined buffer kind limit")
        if buffer["kind"] == "receipt":
            receipt += size
        else:
            logical += size
    need(logical <= limits["aggregate_bytes"] and receipt <= limits["receipt_bytes_separate"], "logical aggregate limit")
    reserve = budget["reserved_physical_bytes"]
    need(unsigned(reserve) and reserve >= physical, "actual retained copy was not reserved")


def binary_verdict(binary, target, schema_ref, prefix, wire):
    run_target = target if target in {"closure", "schema_fragment", "canonical_grammar"} else "canonical_grammar"
    try:
        result = subprocess.run([str(binary), run_target, schema_ref or "", prefix], input=wire,
                                stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL,
                                timeout=5, check=False)
    except (OSError, subprocess.TimeoutExpired) as error:
        raise HarnessError(f"offline parser invocation failed: {error}") from error
    if result.returncode not in (0, 81):
        raise HarnessError(f"offline parser returned {result.returncode}, expected0/81")
    return result.returncode


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", type=Path, required=True)
    parser.add_argument("--schema", type=Path, default=Path("tests/fixtures/dream-proof-tools/closure-boundary.schema.json"))
    parser.add_argument("--vectors", type=Path, default=Path("tests/fixtures/dream-proof-tools/closure-boundary-vectors.json"))
    args = parser.parse_args()
    if not args.binary.is_absolute():
        raise HarnessError("--binary must name an absolute offline parser wrapper")
    binary = args.binary.resolve(strict=True)
    binary_before = digest(binary.read_bytes())
    schema = pinned_json(args.schema, SCHEMA_SHA, 15157)
    corpus = pinned_json(args.vectors, VECTORS_SHA, 3900950)
    # Outside the immutable corpus: split rows cannot evade a per-kind cap,
    # while retained physical copies are charged without duplicating logical IO.
    cap = schema["x-budgets"]["basis_bytes"]
    split = {"buffers": [{"kind": "basis", "copies": 1, "wire_hex": (b"x" * (cap // 2 + 1)).hex()}] * 2,
             "reserved_physical_bytes": cap + 2}
    try:
        check_budget(split, schema["x-budgets"])
    except Refusal:
        pass
    else:
        raise HarnessError("split buffer causal control escaped per-kind bound")

    if corpus["format"] != "bboxy.dream.closure-boundary-vectors/1" or corpus["schema_sha256"] != SCHEMA_SHA:
        raise HarnessError("corpus format/schema binding differs")
    contexts = {}
    for record in corpus["contexts"]:
        if record["id"] in contexts or digest(canonical(record["value"])) != record["sha256"]:
            raise HarnessError("context ID/digest differs")
        try:
            validate_context(record["value"])
        except (Refusal, KeyError, TypeError) as error:
            raise HarnessError(f"invalid signed synthetic context {record['id']}: {error}") from error
        contexts[record["id"]] = record["value"]
    vectors = corpus["vectors"]
    if len(vectors) != 324 or len(contexts) != 13 or Counter(v["expected"] for v in vectors) != {"accept": 44, "reject": 280}:
        raise HarnessError("signed corpus inventory differs")
    failures, errors = [], []
    ids, counts, parser_counts = set(), Counter(), Counter()
    for vector in vectors:
        try:
            vector_id, target = vector["id"], vector["target"]
            if vector_id in ids or target not in TARGETS:
                raise HarnessError("duplicate vector ID or unknown target")
            ids.add(vector_id)
            wire = unhex(vector["wire_hex"])
            if not unsigned(vector["wire_len"]) or len(wire) != vector["wire_len"] or digest(wire) != vector["wire_sha256"]:
                raise HarnessError("vector wire digest/length differs")
            context_id = vector["context_id"]
            if context_id is not None and context_id not in contexts:
                raise HarnessError("vector context missing")
            context = contexts.get(context_id)
            code = binary_verdict(binary, target, vector["schema_ref"], context["prefix"] if context else "", wire)
            parser_counts[f"{target}:{code}"] += 1
            accepted, reason = code == 0, "C parser refused"
            # Context failures must not hide C acceptance of the corpus's
            # structural/syntax negative controls. Context-only mutations can
            # legitimately pass shape and then fail their separately frozen facts.
            structural_negative = (vector["expected"] == "reject" and target in {"closure", "schema_fragment", "canonical_grammar"}
                                   and vector["invariant"] not in CONTEXTUAL_INVARIANTS
                                   and vector_id != "changed-file-hash")
            if code == 0 and structural_negative:
                raise HarnessError("C parser accepted a structural/syntax negative control")
            if accepted:
                try:
                    value = load_json(wire)
                except (ValueError, UnicodeError) as error:
                    raise HarnessError("C parser accepted invalid JSON") from error
                try:
                    if target == "closure":
                        need(context is not None, "closure has no frozen context")
                        closure_context(value, context, vector)
                    elif target == "member_framing":
                        framed_members(unhex(vector["member_frame_hex"]))
                    elif target == "byte_accounting":
                        check_budget(vector["budget"], schema["x-budgets"])
                    reason = "accepted"
                except (Refusal, KeyError, TypeError) as error:
                    accepted, reason = False, str(error)
            actual = "accept" if accepted else "reject"
            counts[f"{target}:{actual}"] += 1
            if actual != vector["expected"]:
                failures.append({"id": vector_id, "expected": vector["expected"], "actual": actual,
                                 "parser_exit": code, "detail": reason})
        except HarnessError as error:
            errors.append({"id": vector.get("id", "<missing>"), "error": str(error)})
    if digest(binary.read_bytes()) != binary_before:
        errors.append({"id": "<parser>", "error": "offline parser bytes changed during run"})
    summary = {"format": "bboxy.dream.closure-vector-check/1", "scope": "offline_synthetic_codec_only",
               "schema_sha256": SCHEMA_SHA, "vectors_sha256": VECTORS_SHA, "binary_sha256": binary_before,
               "vectors": len(vectors), "evaluated": sum(counts.values()), "contexts": len(contexts),
               "counts": dict(sorted(counts.items())), "parser_counts": dict(sorted(parser_counts.items())),
               "failures": failures, "errors": errors, "limitations": LIMITATIONS}
    print(json.dumps(summary, sort_keys=True, separators=(",", ":")))
    return 2 if errors else 1 if failures else 0


if __name__ == "__main__":
    try:
        sys.exit(main())
    except (HarnessError, OSError, ValueError, KeyError, TypeError) as error:
        print(f"closure vector harness error: {error}", file=sys.stderr)
        sys.exit(2)
