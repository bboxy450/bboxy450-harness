# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Black-box gates use throwaway git indexes and never print sentinel values."""
import ast
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest
ROOT=Path(__file__).resolve().parents[1]
class Fixture(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory(prefix='harness-check-',dir='/tmp');self.root=Path(self.temp.name)
        (self.root/'scripts').mkdir();shutil.copy(ROOT/'scripts/checks.py',self.root/'scripts/checks.py');shutil.copy(ROOT/'ATTRIBUTION.md',self.root/'ATTRIBUTION.md')
        (self.root/'Cargo.toml').write_text('\n'.join(l.replace('//','#',1) for l in (ROOT/'src/lib.rs').read_text().splitlines()[:6])+'\n[package]\nname="fixture"\n')
        subprocess.run(['git','init','-q',str(self.root)],check=True)
    def tearDown(self):self.temp.cleanup()
    def gate(self,kind):return subprocess.run(['python3','-I','scripts/checks.py',kind],cwd=self.root,text=True,capture_output=True)
    def test_secret_scan_covers_tracked_root_and_arbitrary_directories(self):
        for path in ['README.md','Cargo.toml','assets/nested/notes.txt','assets/opaque.bin']:
            with self.subTest(path=path):
                file=self.root/path;file.parent.mkdir(parents=True,exist_ok=True);file.write_bytes((b'\xff' if file.suffix=='.bin' else b'')+('api_key: '+ 'sk-'+ 'A'*40+'\n').encode())
                subprocess.run(['git','add','--',path],cwd=self.root,check=True)
                r=self.gate('secrets');self.assertNotEqual(r.returncode,0,'tracked sentinel was missed');self.assertIn(path,r.stderr);self.assertNotIn('A'*40,r.stdout+r.stderr)
                file.write_text('safe\n')
        self.assertEqual(self.gate('secrets').returncode,0)
    def test_cargo_manifest_requires_its_comment_header(self):
        p=self.root/'Cargo.toml';p.write_text('[package]\nname="fixture"\n');self.assertNotEqual(self.gate('headers').returncode,0)
        header='\n'.join(l.replace('//','#',1) for l in (ROOT/'src/lib.rs').read_text().splitlines()[:6])+'\n';p.write_text(header+p.read_text());self.assertEqual(self.gate('headers').returncode,0)
    def test_verification_toml_requires_its_comment_header(self):
        p=self.root/'verification/coverage.toml';p.parent.mkdir();p.write_text('crate=[]\n');self.assertNotEqual(self.gate('headers').returncode,0)
        header='\n'.join(l.replace('//','#',1) for l in (ROOT/'src/lib.rs').read_text().splitlines()[:6])+'\n';p.write_text(header+p.read_text());self.assertEqual(self.gate('headers').returncode,0)
class TestStack(unittest.TestCase):
    def test_native_target_never_resolves_reference_tools(self):
        r=subprocess.run(['make','-n','test-formal-native'],cwd=ROOT,text=True,capture_output=True,check=True)
        self.assertIn('formal.py --native',r.stdout);self.assertNotIn('scripts/test-formal.py',r.stdout);self.assertNotIn('formal.py --target',r.stdout)
    def test_canonical_stack_names_both_backends_native_first(self):
        tree=ast.parse((ROOT/'scripts/test.py').read_text());commands=[]
        for node in ast.walk(tree):
            if isinstance(node,ast.For) and isinstance(node.target,ast.Name) and node.target.id=='command':commands=ast.literal_eval(node.iter)
        native=['make','test-formal-native'];parity=['make','test-formal-parity']
        self.assertIn(native,commands);self.assertIn(parity,commands);self.assertLess(commands.index(native),commands.index(parity));self.assertIn(['python3','-I','scripts/test-hardening.py'],commands)
if __name__=='__main__':unittest.main()
