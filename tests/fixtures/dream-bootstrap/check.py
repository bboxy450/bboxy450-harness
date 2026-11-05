#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Independently check immutable dream-bootstrap/1 binary/binding vectors.

No runtime launch, source access, deadline-admission claim or fixture regeneration.
The fixed deadline requires an injected CLOCK_MONOTONIC value at runtime.
"""
import hashlib
import json
import pathlib
import re
import struct
import sys

ROOT = pathlib.Path(__file__).resolve().parent
ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ"
CONTEXT_KEYS = {
    "account", "account_epoch", "seat", "generation", "grant_epoch", "swarm",
    "swarm_revision", "workspace", "workspace_revision", "root", "root_revision",
    "provider", "provider_epoch",
}
REQUIRED_CASES = {
    "inspect", "inspect-env-leading-zero", "inspect-probe", "read-prepare", "magic", "purpose-zero",
    "purpose-three", "probe-two", "reserved-first", "reserved-last",
    "tail-first", "tail-last", "incarnation-zero", "sequence-zero",
    "sequence-two", "truncated", "extra", "stale-env-birth", "env-binding",
    "record-binding", "wrong-run", "wrong-nonce", "wrong-purpose",
    "wrong-probe", "wrong-incarnation", "env-version", "env-fd-low",
    "env-fd-high", "env-fd-sign", "env-extra", "env-upper-hash",
    "env-lower-birth", "env-overflow-birth", "env-overlength",
}
BINDING_KEYS = {
    "profile", "purpose", "probe", "op_id", "run_id", "nonce", "incarnation",
    "context", "worker_sha256",
}


def require(condition, message):
    if not condition:
        raise ValueError(message)


def canonical(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True,
                      separators=(",", ":"), allow_nan=False).encode("utf-8")


def ulid_bytes(text):
    require(isinstance(text, str) and len(text) == 26, "ULID length")
    value = 0
    for char in text:
        require(char in ALPHABET, "ULID alphabet/case")
        value = value * 32 + ALPHABET.index(char)
    require(value < 2 ** 128, "ULID overflow")
    return value.to_bytes(16, "big")


def ulid_text(raw):
    require(len(raw) == 16, "ULID binary length")
    number = int.from_bytes(raw, "big")
    result = ""
    for _ in range(26):
        result = ALPHABET[number & 31] + result
        number >>= 5
    return result


def decode(raw):
    require(len(raw) == 128, "binary length")
    require(raw[:8] == b"BBDBOOT1", "magic")
    require(raw[8] in (1, 2), "purpose")
    require(raw[9] in (0, 1), "probe")
    require(raw[10:16] == bytes(6) and raw[120:128] == bytes(8), "reserved")
    incarnation, sequence, deadline = struct.unpack_from("<QQQ", raw, 96)
    require(incarnation != 0, "incarnation")
    require(sequence == 1, "sequence")
    return {
        "purpose": ("inspect-c1" if raw[8] == 1 else "read-prepare"),
        "probe": bool(raw[9]),
        "birth": ulid_text(raw[16:32]),
        "run_id": ulid_text(raw[32:48]),
        "nonce": ulid_text(raw[48:64]),
        "binding_sha256": raw[64:96].hex(),
        "incarnation": incarnation,
        "sequence": sequence,
        "deadline_ns": deadline,
    }


def environment(text):
    require(isinstance(text, str) and text.isascii() and len(text) <= 128, "environment bytes")
    fields = text.split(":")
    require(len(fields) == 4 and fields[0] == "1", "environment version/fields")
    require(re.fullmatch(r"[0-9]+", fields[1]) is not None, "descriptor digits")
    fd = int(fields[1])
    require(3 <= fd <= 65535, "descriptor range")
    ulid_bytes(fields[2])
    require(re.fullmatch(r"[0-9a-f]{64}", fields[3]) is not None, "binding hex")
    return {"fd": fd, "birth": fields[2], "binding_sha256": fields[3]}


def succeeds(fn, value):
    try:
        return True, fn(value)
    except ValueError:
        return False, None


def fixture_file(relative):
    path = ROOT / relative
    require(path.resolve().is_relative_to(ROOT), "fixture escape")
    return path.read_bytes()


def validate_binding_inputs(inputs):
    require(isinstance(inputs, dict) and set(inputs) == BINDING_KEYS, "binding keys")
    require(inputs["profile"] == "dream-bootstrap/1", "binding profile")
    require(inputs["purpose"] in ("inspect-c1", "read-prepare"), "binding purpose")
    require(type(inputs["probe"]) is bool, "binding probe")
    require(type(inputs["incarnation"]) is int and 0 < inputs["incarnation"] < 2 ** 64,
            "binding incarnation")
    for key in ("op_id", "run_id", "nonce"):
        ulid_bytes(inputs[key])
    context = inputs["context"]
    if inputs["probe"]:
        require(context is None, "probe requires explicit context:null")
    else:
        require(isinstance(context, dict) and set(context) == CONTEXT_KEYS,
                "work requires complete C1 context")
        for key, value in context.items():
            if key in ("account", "workspace", "root", "provider"):
                require(isinstance(value, str) and value and "\0" not in value, "context identifier")
            elif key in ("seat", "swarm"):
                ulid_bytes(value)
            else:
                require(type(value) is int and 0 < value <= 2 ** 53 - 1, "context revision")
    require(re.fullmatch(r"[0-9a-f]{64}", inputs["worker_sha256"]) is not None, "worker hash")


def check_binding(authority):
    inputs = authority["inputs"]
    validate_binding_inputs(inputs)
    ulid_bytes(authority["birth"])
    encoded = canonical(inputs)
    require(encoded.decode("utf-8") == authority["binding_canonical_json"], "canonical binding bytes")
    digest = hashlib.sha256(encoded).hexdigest()
    require(digest == authority["binding_sha256"], "binding digest")
    return digest


def check():
    inventory = json.loads((ROOT / "vectors.json").read_text(encoding="utf-8"))
    require(inventory["profile"] == "dream-bootstrap/1", "inventory profile")
    require(inventory["record_bytes"] == 128, "inventory record size")
    # Fixed independent ULID anchors detect an encoder/decoder that agrees only
    # with itself. 01ARZ3NDEKTSV4RRFFQ69G5FAV is the canonical C1 fixture ID.
    require(ulid_bytes("00000000000000000000000000") == bytes(16), "zero ULID anchor")
    require(ulid_bytes("7ZZZZZZZZZZZZZZZZZZZZZZZZZ") == bytes([255]) * 16, "max ULID anchor")
    require(ulid_bytes("01ARZ3NDEKTSV4RRFFQ69G5FAV").hex()
            == "01563e3ab5d3d6764c61efb99302bd5b", "canonical C1 ULID anchor")
    binding_seen = set()
    for case in inventory["binding_cases"]:
        name = case["name"]
        require(name not in binding_seen, "duplicate binding case")
        binding_seen.add(name)
        encoded = canonical(case["inputs"])
        require(encoded.decode("utf-8") == case["binding_canonical_json"], name + ": binding case bytes")
        require(hashlib.sha256(encoded).hexdigest() == case["binding_sha256"], name + ": binding case hash")
        valid, _ = succeeds(validate_binding_inputs, case["inputs"])
        require(valid == (case["expect"] == "valid"), name + ": binding input verdict")
    require(binding_seen == {
        "probe-null", "work-full", "probe-nonnull", "probe-empty-context",
        "probe-missing-context", "work-null-context", "work-missing-context",
        "work-empty-context",
    }, "required probe/work binding corpus changed or reduced")
    require(len(binding_seen) == inventory["binding_case_count"], "binding case count")
    seen = set()
    counts = {"valid": 0, "invalid_binary": 0, "invalid_environment": 0, "mismatch": 0}
    for case in inventory["cases"]:
        name = case["name"]
        require(name not in seen, "duplicate case")
        seen.add(name)
        authority = case["authority"]
        digest = check_binding(authority)
        if name in ("inspect", "inspect-probe", "read-prepare"):
            require(fixture_file(case["binding_file"]) == canonical(authority["inputs"]),
                    name + ": canonical sidecar bytes")
            require(fixture_file(case["binding_hash_file"]) == (digest + "\n").encode("ascii"),
                    name + ": sidecar digest bytes")
        raw = fixture_file(case["file"])
        require(hashlib.sha256(raw).hexdigest() == case["record_sha256"], name + ": record digest")
        binary_ok, record = succeeds(decode, raw)
        env_ok, env = succeeds(environment, case["environment"])
        require(binary_ok == (case["expect"]["binary_format"] == "valid"), name + ": binary verdict")
        require(env_ok == (case["expect"]["environment_format"] == "valid"), name + ": environment verdict")
        if not binary_ok:
            require(case["expect"]["identity"] == "not_checked", name + ": invalid binary identity")
            counts["invalid_binary"] += 1
            continue
        if not env_ok:
            require(case["expect"]["identity"] == "not_checked", name + ": invalid environment identity")
            counts["invalid_environment"] += 1
            continue
        inputs = authority["inputs"]
        matches = (record["birth"] == env["birth"] == authority["birth"]
                   and record["binding_sha256"] == env["binding_sha256"] == digest
                   and record["run_id"] == inputs["run_id"]
                   and record["nonce"] == inputs["nonce"]
                   and record["purpose"] == inputs["purpose"]
                   and record["probe"] == inputs["probe"]
                   and record["incarnation"] == inputs["incarnation"]
                   and record["deadline_ns"] == inventory["static_deadline_ns"]
                   and env["fd"] == authority["fd"])
        require(matches == (case["expect"]["identity"] == "match"), name + ": identity verdict")
        counts["valid" if matches else "mismatch"] += 1
    require(counts["valid"] == 4, "three binary purpose/probe vectors and leading-zero descriptor alias required")
    require(counts["invalid_binary"] >= 12 and counts["invalid_environment"] >= 6
            and counts["mismatch"] >= 4, "negative corpus reduced")
    require(seen == REQUIRED_CASES, "required named corpus changed or reduced")
    require(len(seen) == inventory["case_count"], "case count")
    print("dream-bootstrap/1 immutable vectors:", len(seen), counts,
          "binding cases:", len(binding_seen),
          "static deadline; runtime time-validity requires injected clock")


if __name__ == "__main__":
    try:
        check()
    except (ValueError, KeyError, TypeError, OSError, json.JSONDecodeError) as error:
        print("dream-bootstrap fixture failure:", error, file=sys.stderr)
        sys.exit(1)
