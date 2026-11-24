# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Black-box gates use throwaway git indexes and never print sentinel values."""
import ast
from contextlib import ExitStack
import json
import os
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
        control=['python3','-I','scripts/test-capability-control.py'];self.assertIn(control,commands);self.assertLess(commands.index(control),commands.index(native))

    def test_land_requires_formal_evidence(self):
        with tempfile.TemporaryDirectory(prefix="harness-land-control-", dir="/tmp") as private, ExitStack() as cleanup:
            base = Path(private)
            checkout, upstream, remote = (base / name for name in
                                          ("harness checkout", "bboxy450-term", "origin"))
            gnupg, bin_dir = base / "gnupg", base / "bin"
            gnupg.mkdir(mode=0o700)
            bin_dir.mkdir()
            env = {name: value for name, value in os.environ.items()
                   if not name.startswith(("GIT_", "BBOXY_"))
                   and name not in {"MAKEFLAGS", "MFLAGS", "GNUMAKEFLAGS", "MAKELEVEL"}}
            env.update(HOME=private, GNUPGHOME=str(gnupg), GIT_CONFIG_NOSYSTEM="1",
                       GIT_CONFIG_GLOBAL=os.devnull, PATH=f"{bin_dir}:{env['PATH']}")

            def run(*args, cwd=checkout):
                return subprocess.check_output(args, cwd=cwd, env=env, text=True,
                                               stderr=subprocess.STDOUT).strip()

            for repo in (checkout, upstream):
                run("git", "init", "-q", "-b", "main", str(repo), cwd=base)
                run("git", "config", "user.name", "Harness land control", cwd=repo)
                run("git", "config", "user.email", "land-control@example.invalid", cwd=repo)
            cleanup.callback(run, "gpgconf", "--kill", "gpg-agent", cwd=base)
            run("gpg", "--batch", "--pinentry-mode", "loopback", "--passphrase", "",
                "--quick-generate-key", "land-control@example.invalid", "ed25519", "sign", "1d")
            run("git", "config", "user.signingkey", "land-control@example.invalid")

            pins = json.loads((ROOT / "verification/upstream-sources.json").read_text())
            for pin in pins["pins"]:
                snapshot = (ROOT / pin["snapshot"]).read_bytes()
                for path, content in ((checkout / pin["snapshot"], snapshot),
                                      (upstream / pin["source"], snapshot.split(b"\n", 6)[6])):
                    path.parent.mkdir(parents=True, exist_ok=True)
                    path.write_bytes(content)
            run("git", "add", ".", cwd=upstream)
            run("git", "commit", "-qm", "Pinned upstream", cwd=upstream)
            upstream_formal = upstream / "scripts/formal.py"
            matching_source = upstream_formal.read_bytes()
            upstream_formal.write_bytes(matching_source + b"\n# stale control\n")
            run("git", "commit", "-qam", "Change upstream formal source", cwd=upstream)

            for name in ("Makefile", "scripts/land.py", "verification/upstream-sources.json",
                         "tests/gui_rich.rs", "tests/gui_render.rs", "tests/gui_card_bounds.rs",
                         "tests/mcp_client.rs"):
                path = checkout / name
                path.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(ROOT / name, path)
            shutil.copyfile(ROOT / "scripts/formal.py", checkout / "scripts/real-formal.py")
            # Formal source pin and evidence rejection are real. Builds and the
            # parity producer are isolated so the missing-receipt case is deterministic.
            (checkout / "scripts/formal.py").write_text(
                'import os, runpy, sys\n'
                'if "--verify-evidence" in sys.argv or os.getenv("STALE_FORMAL"):\n'
                '    runpy.run_path("scripts/real-formal.py", run_name="__main__")\n'
            )
            for name in ("checks.py", "suite-licenses.py", "test-suite-licenses.py", "test-formal.py"):
                (checkout / "scripts" / name).touch()
            cargo = bin_dir / "cargo"
            cargo.write_text("#!/bin/sh\nprintf 'running 1 test\\n'\n")
            cargo.chmod(0o755)

            run("git", "add", ".")
            run("git", "commit", "-qm", "Harness fixture base")
            run("git", "clone", "-q", "--bare", str(checkout), str(remote))
            run("git", "remote", "add", "origin", str(remote))
            initial = run("git", "rev-parse", "main", cwd=remote)
            run("git", "commit", "-qS", "--allow-empty", "-m", "Signed landing candidate")

            for stale, failed, cause in (
                (True, "test-formal-parity", "upstream main SOURCE hash changed; re-copy scripts/formal.py"),
                (False, "check-formal-evidence", "No such file or directory"),
            ):
                with self.subTest(stage=failed):
                    if stale:
                        env["STALE_FORMAL"] = "1"
                    else:
                        env.pop("STALE_FORMAL")
                        upstream_formal.write_bytes(matching_source)
                        run("git", "commit", "-qam", "Restore pinned upstream", cwd=upstream)
                    target = base / failed
                    result = subprocess.run(["make", "land", f"TARGET_DIR={target}"], cwd=checkout,
                                            env=env, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT)
                    self.assertEqual(result.returncode, 2, result.stdout)
                    self.assertIn("formal: FAILED:", result.stdout)
                    self.assertIn(cause, result.stdout)
                    self.assertIn(f"land: gate failed: make {failed} ", result.stdout)
                    receipt = json.loads((target / "harness-land-receipt.json").read_text())
                    formal_stages = {"test-formal-parity", "check-formal-evidence"}
                    stages = [row["argv"][1] for row in receipt["commands"]
                              if row["argv"][0] == "make" and row["argv"][1] in formal_stages]
                    expected = ["test-formal-parity"] if stale else ["test-formal-parity", failed]
                    self.assertEqual(stages, expected)
                    self.assertEqual(receipt["commands"][-1]["exit"], 2)
                    self.assertNotIn("land: git push ", result.stdout)
                    self.assertNotIn("landed", receipt)
                    self.assertEqual(run("git", "rev-parse", "main", cwd=remote), initial)

if __name__=='__main__':unittest.main()
