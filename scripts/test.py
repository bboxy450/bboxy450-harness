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
    target = Path(env.get('TARGET_DIR', env.get('CARGO_TARGET_DIR', 'target')))
    target = (ROOT / target).resolve()
    env['CARGO_TARGET_DIR'] = env['TARGET_DIR'] = str(target)
    env['DREAM_C1_RUNNER'] = str(target / 'dream-c1-native-profile/worker')
    env['DREAM_READ_RUNNER'] = str(target / 'dream-read-native-profile/worker')
    # These witnesses hash and launch source-bound ordinary workers. Reuse
    # the optimized serial lane used by test-dream-c1 and test-dream-mcp.
    dream_process_cases = [
        'actual_store_fork_is_explicitly_unsupported_and_parent_rows_never_pass',
        'bootstrap_held_birth_has_zero_admission_and_independent_probe_then_real_completion',
        'four_children_keep_independent_controls_and_individual_memory_shares',
        'near_limit_rejected_envelope_drains_before_process_done_and_join',
        'paused_owner_does_not_delay_another_process_and_owned_kill_never_completes',
        'real_worker_reuses_failure_retry_and_denial_fingerprints',
        'source_blob_sequence_version_and_custody_controls',
        'no_candidates_protocol_pause_cancel_and_descriptor_custody_are_separate',
        'read_bootstrap_held_birth_has_zero_admission_then_paid_hello_start_and_actual_reap',
        'read_bootstrap_probe_null_context_permanently_refuses_matching_start_before_source',
        'selected_dense_owner_and_total_input_overflow_refuse_before_publication',
        'selected_multi_mib_four_jobs_keep_control_and_measured_storage_limits',
        'stdout_backpressure_and_activity_flood_leave_cancel_independent',
    ]
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
            ['cargo','test','--offline','--locked','--all-targets','--','--skip','non_pty_spawn_does_not_keep_a_sibling_master_alive','@SKIP_DREAM_PROCESS_CASES@'],
            # Keep launcher process bursts separate from PTY lifecycle deadlines.
            # Every case still runs; neither the deadline nor product scheduling changes.
            ['cargo','test','--offline','--locked','--all-targets','--features','acp,test-fixtures','--','--skip','seat_launch::','--skip','non_pty_spawn_does_not_keep_a_sibling_master_alive','@SKIP_DREAM_PROCESS_CASES@'],
            ['cargo','test','--offline','--locked','--features','acp,test-fixtures','--test','seat_process','seat_launch::'],
            # This case deliberately clears CLOEXEC. Its test process must have
            # no concurrent test that could fork while the master is exposed.
            ['cargo','test','--offline','--locked','--features','acp,test-fixtures','--test','seat_process','non_pty_spawn_does_not_keep_a_sibling_master_alive','--','--exact'],
            ['make','prepare-dream-c1-worker','prepare-dream-read-worker'],
            ['cargo','test','--offline','--locked','--release','--test','dream_c1','--test','dream_read','--','--test-threads=1'],
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
            if command[-1] == '@SKIP_DREAM_PROCESS_CASES@':
                command = command[:-1]
                for case in dream_process_cases:
                    command.extend(['--skip', case])
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
