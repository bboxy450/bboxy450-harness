# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Canonical gate with a private HOME and explicit command/exit records."""
import hashlib
import datetime
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
ROOT=Path(__file__).resolve().parents[1]
def source_hashes():
    paths=[p for base in ['src','tests','scripts'] for p in (ROOT/base).rglob('*') if p.is_file() and '__pycache__' not in p.parts]
    paths += [ROOT/p for p in ['Cargo.toml','Cargo.lock','Makefile','verification/corpus.toml','verification/coverage.toml','verification/contracts.toml','verification/tools.lock.json']]
    return {str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted(paths)}
def main():
    env=os.environ.copy()
    original=Path(env['HOME'])
    env['CARGO_HOME']=env.get('CARGO_HOME',str(original/'.cargo'))
    env['RUSTUP_HOME']=env.get('RUSTUP_HOME',str(original/'.rustup'))
    env.pop('BBOXY_CAPABILITY',None)
    env.pop('BBOXY_CAPABILITY_FILE',None)
    # Test sessions must not announce themselves as the operator's live seat.
    # PTY lifecycle fixtures supply their own disposable seat identity.
    env.pop('BBOXY_SEAT_ID',None)
    inputs=source_hashes()
    rows=[]
    with tempfile.TemporaryDirectory(prefix='harness-tests-',dir='/tmp') as own:
        env['HOME']=own
        env['TMPDIR']=own
        for command in [
            ['cargo','fmt','--check'],
            ['make','check-embedded-build'],
            ['cargo','clippy','--offline','--locked','--all-targets','--keep-going','--','-D','warnings'],
            ['cargo','clippy','--offline','--locked','--all-targets','--all-features','--keep-going','--','-D','warnings'],
            ['cargo','test','--offline','--locked','--all-targets','--','--skip','non_pty_spawn_does_not_keep_a_sibling_master_alive'],
            # Keep launcher process bursts separate from PTY lifecycle deadlines.
            # Every case still runs; neither the deadline nor product scheduling changes.
            ['cargo','test','--offline','--locked','--all-targets','--features','acp,test-fixtures','--','--skip','seat_launch::','--skip','non_pty_spawn_does_not_keep_a_sibling_master_alive'],
            ['cargo','test','--offline','--locked','--features','acp,test-fixtures','--test','seat_process','seat_launch::'],
            # This case deliberately clears CLOEXEC. Its test process must have
            # no concurrent test that could fork while the master is exposed.
            ['cargo','test','--offline','--locked','--features','acp,test-fixtures','--test','seat_process','non_pty_spawn_does_not_keep_a_sibling_master_alive','--','--exact'],
            ['python3','-I','scripts/test-hardening.py'],
            ['python3','-I','scripts/test-capability-control.py'],
            ['python3','-I','scripts/test-socket-heartbeat-control.py'],
            ['python3','-I','scripts/test-launch-parity-control.py'],
            ['python3','-I','scripts/test-launch-approval-control.py'],
            ['make','test-formal-native'],
            ['make','test-formal-parity'],
            ['python3','-I','scripts/checks.py','secrets'],
            ['python3','-I','scripts/checks.py','secrets-control'],
            ['make','test-licenses'],
            ['python3','-I','scripts/checks.py','kb-records'],
            ['python3','-I','scripts/checks.py','kb-records-control'],
            ['python3','-I','scripts/checks.py','inventory-shape'],
            ['python3','-I','scripts/checks.py','inventory-shape-control'],
            # protocol.schema.json is regenerated and compared by tests/protocol_golden.rs above
            ['make','test-ir'],
            ['cargo','run','--offline','--locked','--','ir-scan','--propose','@TEST_HOME@/ir-proposals','--check-pending','tests/fixtures/ir-v1/pending','tests/fixtures/'],
        ]:
            command=[arg.replace('@TEST_HOME@',own) for arg in command]
            print('test: '+ ' '.join(command),flush=True)
            result=subprocess.run(command,cwd=ROOT,env=env)
            rows.append({'command':command,'exit_code':result.returncode})
            if result.returncode:break
    code=rows[-1]['exit_code']
    if inputs!=source_hashes():code=1
    record={'at':datetime.datetime.now(datetime.timezone.utc).isoformat(),'source':subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),'input_sha256':inputs,'private_home':True,'operator_input':False,'outcome':'PASS' if code==0 else 'FAIL','commands':rows}
    (ROOT/'.bboxy/runs').mkdir(parents=True,exist_ok=True)
    (ROOT/'.bboxy/runs/test-current.json').write_text(json.dumps(record,indent=2)+'\n')
    return code
if __name__=='__main__':sys.exit(main())
