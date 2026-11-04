#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Adapted from bboxy450-harness scripts/verify-harness-onboarding.py: focused named private checks.
"""Named private clean-start process, native GUI/Bridge and affected compile checks."""
import hashlib, json, os, subprocess, sys, tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[1]
TARGET=Path(os.environ.get('TARGET_DIR',ROOT/'target')).resolve()
OUT=Path(tempfile.mkdtemp(prefix='bb-clean-start-gate-',dir='/tmp'))
env=dict(os.environ,RUSTC_WRAPPER='',CARGO_BUILD_JOBS='2',CARGO_PROFILE_DEV_DEBUG='0',RUST_TEST_THREADS='2',CLEAN_START_PYTHON=sys.executable,CLIPPY_CONF_DIR=str(ROOT/'src/frontends/gui/lint'))
cargo=['--offline','--locked','--jobs','2','--target-dir',str(TARGET),'-p','bboxy450-harness']
def hashes():
 paths=['Makefile','ATTRIBUTION.md','scripts/verify-clean-start.py','src/onboarding.rs','src/frontends/headless.rs','src/frontends/gui/options.rs','src/frontends/gui/window.rs','src/frontends/gui/bridge.rs','tests/clean_start.rs','tests/support/clean_start.rs','tests/support/onboarding_startup.rs']
 return {name:hashlib.sha256((ROOT/name).read_bytes()).hexdigest() for name in paths}
initial=hashes()
commands=[]
for name,command in [
 ('private-process-and-native-gui',['cargo','nextest','run',*cargo,'--features','standalone','--test','clean_start']),
 ('gui-option-parsing',['cargo','nextest','run',*cargo,'--features','standalone','--test','gui_cli']),
 ('actual-bridge-precedence-and-existing-setup',['cargo','nextest','run',*cargo,'--features','standalone','--lib','-E','test(clean_start_) | test(onboarding_empty_home_live_bridge_refresh)']),
 ('strict-affected-standalone',['cargo','clippy',*cargo,'--features','standalone','--lib','--bin','bboxy450-harness','--test','clean_start','--no-deps','--','-D','warnings']),
 ('embedded-without-acp',['cargo','check',*cargo,'--no-default-features','--features','embedded','--lib']),
 ('license-headers',['make','test-licenses']),
]:
 log=OUT/(name+'.log')
 with log.open('w') as stream:
  result=subprocess.run(command,cwd=ROOT,env=env,stdout=stream,stderr=subprocess.STDOUT)
 commands.append(dict(name=name,command=command,exit_code=result.returncode,log=str(log),sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
 print(f'clean-start: {name}: exit {result.returncode}',flush=True)
 if result.returncode:
  print(log.read_text()[-7000:],file=sys.stderr)
  raise SystemExit(result.returncode)
assert initial==hashes(), "Source changed during verification"
receipt=dict(source_hashes=initial,commands=commands,operator_input=False,raw_logs=str(OUT),unchanged_contracts=['protocol','runtime','installer ownership','create-new profile publication'],default_file_written=False)
(OUT/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('clean-start receipt: '+str(OUT/'receipt.json'),flush=True)
if value:=os.environ.get('CLEAN_START_RECEIPT'):
 Path(value).write_text(json.dumps(receipt,indent=2)+'\n')
