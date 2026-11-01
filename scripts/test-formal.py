# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Adapted from bboxy450-term: https://github.com/bboxy450/bboxy450-term/blob/main/scripts/test-formal.py
"""Negative controls for the reused gate; tool failures never count as model failures."""
import sys
sys.dont_write_bytecode=True
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
spec=importlib.util.spec_from_file_location('formal',Path(__file__).with_name('formal.py'))
formal=importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
class Gate(unittest.TestCase):
    def test_tool_error_is_not_a_property_verdict(self):
        for code in [1,2,10,99]:
            with self.assertRaises(ValueError):formal.verdict(subprocess.CompletedProcess([],code,'syntax error'))
    def test_named_diagnostic_requires_real_exit(self):
        with self.assertRaises(ValueError):formal.verdict(subprocess.CompletedProcess([],0,'Invariant is violated'))
        self.assertEqual(formal.verdict(subprocess.CompletedProcess([],12,'Invariant is violated')),'safety')
    def test_tampered_pin_fails(self):
        with tempfile.TemporaryDirectory() as t:
            p=Path(t)/'jar';p.write_bytes(b'wrong')
            with self.assertRaises(ValueError):formal.verify_digest(p,'0'*64)
    def test_omitted_crate_and_source_fail(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t);(root/'verification').mkdir();(root/'src').mkdir();(root/'Cargo.toml').write_text('[package]\nname="harness"\n');(root/'verification/coverage.toml').write_text('crate=[]')
            with self.assertRaises(ValueError):formal.coverage(root,set())
    def test_unknown_dot_and_refinement_claim_fail(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t);(root/'bad.dot').write_text('garbage')
            with self.assertRaises(ValueError):formal.dot_graph(root/'bad.dot',1)
            (root/'X.metadata.json').write_text(json.dumps({'variables':[{'min':0,'max':1,'initial':0}],'actions':['A'],'invariants':['I'],'implementation_refinement_proven':True}))
            with self.assertRaises(ValueError):formal.contract_metadata(root,'X')
    def test_source_snapshots(self):formal.source_pins()
if __name__=='__main__':unittest.main()
