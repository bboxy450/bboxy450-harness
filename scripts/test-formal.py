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
from unittest.mock import patch
spec=importlib.util.spec_from_file_location('formal',Path(__file__).with_name('formal.py'))
formal=importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
class Gate(unittest.TestCase):
    def test_embedded_term_witness_keeps_actual_source_validation(self):
        with tempfile.TemporaryDirectory() as own, patch.dict(formal.os.environ,{},clear=True):
            root=Path(own);child=root/'crates/bboxy450-harness';child.mkdir(parents=True)
            (root/'.gitmodules').write_text('')
            witness=root/'crates/bboxy450-term/src/rich_pane.rs'
            witness.parent.mkdir(parents=True);witness.write_text('private witness')
            path='../bboxy450-term/crates/bboxy450-term/src/rich_pane.rs'
            resolved=formal.implementation_reference(child,path)
            self.assertEqual((child/resolved).resolve(),witness.resolve())
            missing=path.replace('rich_pane.rs','missing.rs')
            self.assertEqual(formal.implementation_reference(child,missing),missing)
    def test_explicit_toolkit_witness_is_authoritative_and_missing_stays_missing(self):
        with tempfile.TemporaryDirectory() as own:
            root=Path(own);child=root/'harness';child.mkdir()
            toolkit=root/'chosen-toolkit';witness=toolkit/'crates/bboxy450-term/src/rich_pane.rs'
            witness.parent.mkdir(parents=True);witness.write_text('chosen witness')
            path='../bboxy450-term/crates/bboxy450-term/src/rich_pane.rs'
            fallback=root/'bboxy450-term/crates/bboxy450-term/src/rich_pane.rs'
            fallback.parent.mkdir(parents=True);fallback.write_text('other witness')
            with patch.dict(formal.os.environ,{'BBOXY_TOOLKIT_ROOT':str(toolkit)}):
                resolved=formal.implementation_reference(child,path)
                self.assertEqual((child/resolved).resolve(),witness.resolve())
                witness.unlink()
                resolved=formal.implementation_reference(child,path)
                self.assertFalse((child/resolved).is_file(),'missing explicit witness must fail validation')
                self.assertNotEqual((child/resolved).resolve(),fallback.resolve())
    def test_managed_module_worktree_resolves_real_source_and_refuses_missing(self):
        with tempfile.TemporaryDirectory() as own:
            root=Path(own);toolkit=root/'source-term';toolkit.mkdir()
            (toolkit/'.gitmodules').write_text('')
            common=toolkit/'.git/modules/crates/bboxy450-harness'
            subprocess.run(['git','init','--bare','--quiet',str(common)],check=True,
                env={'PATH':formal.os.defpath,'HOME':str(root),'GIT_CONFIG_NOSYSTEM':'1'})
            child=root/'managed/checkouts/worker';child.mkdir(parents=True)
            gitdir=common/'worktrees/worker';gitdir.mkdir(parents=True)
            (gitdir/'commondir').write_text('../..\n')
            (gitdir/'HEAD').write_text('ref: refs/heads/worker\n')
            (gitdir/'gitdir').write_text(str(child/'.git')+'\n')
            (child/'.git').write_text('gitdir: '+str(gitdir)+'\n')
            witness=toolkit/'crates/bboxy450-term/src/rich_pane.rs'
            witness.parent.mkdir(parents=True);witness.write_text('actual managed witness')
            path='../bboxy450-term/crates/bboxy450-term/src/rich_pane.rs'
            with patch.dict(formal.os.environ,{},clear=True):
                resolved=formal.implementation_reference(child,path)
                self.assertEqual((child/resolved).resolve(),witness.resolve())
                missing=path.replace('rich_pane.rs','missing.rs')
                self.assertEqual(formal.implementation_reference(child,missing),missing)
    def test_unavailable_git_metadata_cannot_create_a_witness(self):
        with tempfile.TemporaryDirectory() as own:
            path='../bboxy450-term/crates/bboxy450-term/src/rich_pane.rs'
            with patch.dict(formal.os.environ,{},clear=True), patch.object(
                    formal.subprocess,'check_output',side_effect=subprocess.TimeoutExpired('git',2)):
                self.assertEqual(formal.implementation_reference(Path(own),path),path)
    def test_extra_controls_are_explicit_executed_negative_verdicts(self):
        with tempfile.TemporaryDirectory() as t:
            root=Path(t);(root/'verification').mkdir();(root/'witness.rs').write_text('')
            corpus='''[[model]]
name="HarnessExample"
expected="pass"
[[model]]
name="BrokenHarnessExample"
expected="safety"
[[model]]
name="BrokenHarnessExampleProgress"
expected="liveness"
'''
            text='''[[contract]]
name="HarnessExample"
title="example"
models=["HarnessExample","BrokenHarnessExample","BrokenHarnessExampleProgress"]
controls=[{name="BrokenHarnessExample",expected="safety"},{name="BrokenHarnessExampleProgress",expected="liveness"}]
sources=["witness.rs"]
implementation_tests=["witness.rs"]
assumptions="finite"
property="safety and progress"
checks=["make test-formal","make test"]
'''
            (root/'verification/corpus.toml').write_text(corpus)
            path=root/'verification/contracts.toml';path.write_text(text)
            names={'HarnessExample','BrokenHarnessExample','BrokenHarnessExampleProgress'}
            self.assertEqual(len(formal.contracts(root,names)),1)
            for invalid in [
                text.replace('expected="liveness"','expected="pass"'),
                text.replace(',{name="BrokenHarnessExampleProgress",expected="liveness"}',''),
                text.replace('name="BrokenHarnessExampleProgress",expected="liveness"','name="Missing",expected="liveness"'),
            ]:
                path.write_text(invalid)
                with self.assertRaises(ValueError):formal.contracts(root,names)
            path.write_text(text)
            with self.assertRaises(ValueError):formal.contracts(root,names-{'BrokenHarnessExampleProgress'})
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
    def test_tlc_temporary_directories_are_isolated(self):
        with tempfile.TemporaryDirectory() as own:
            root=Path(own)
            with patch.object(formal.subprocess,'run',return_value=subprocess.CompletedProcess([],0,'ok')) as run:
                formal.tlc(Path('/java'),Path('/tlc.jar'),root,'ModelA',True)
                a=run.call_args.args[0][1]
                formal.tlc(Path('/java'),Path('/tlc.jar'),root,'ModelB',True)
                b=run.call_args.args[0][1]
                self.assertTrue(a.startswith('-Djava.io.tmpdir='))
                self.assertNotEqual(a,b)
    def test_source_snapshots(self):formal.source_pins()
if __name__=='__main__':unittest.main()
