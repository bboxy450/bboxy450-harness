#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Check discovery signatures without starting the worker's stdin-driven runtime."""
import ast
import json
from pathlib import Path
import unittest

worker = Path(__file__).resolve().parents[1] / "src/python_execute/worker.py"
function = next(node for node in ast.parse(worker.read_text()).body
                if isinstance(node, ast.FunctionDef) and node.name == "_signature")
namespace = {"json": json}
exec(compile(ast.Module(body=[function], type_ignores=[]), str(worker), "exec"), namespace)
signature = namespace["_signature"]


class Signatures(unittest.TestCase):
    def test_type_unions(self):
        for types, expected in [(["string", "null"], "str | None"),
                                (["integer", "number"], "int | float"),
                                (["array", "object", "boolean"], "list | dict | bool")]:
            with self.subTest(types=types):
                self.assertEqual(signature({"name": "read", "inputSchema": {
                    "properties": {"value": {"type": types}}, "required": ["value"]}}),
                    f"read(value: {expected})")

    def test_scalar_types_and_required_order(self):
        self.assertEqual(signature({"name": "read", "inputSchema": {
            "properties": {"optional": {"type": "string"}, "count": {"type": "integer"},
                           "enabled": {"type": "boolean"}, "payload": {}},
            "required": ["count"]}}),
            "read(count: int, optional: str = ..., enabled: bool = ..., payload: any = ...)")

    def test_enum_precedes_type_union(self):
        self.assertEqual(signature({"name": "read", "inputSchema": {
            "properties": {"value": {"type": ["string", "null"], "enum": ["open", None]}}}}),
            'read(value: "open" | null = ...)')


if __name__ == "__main__":
    unittest.main()
