# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Run canonical synthetic private-engine controls; never run a real candidate."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import re
import stat
import subprocess
import sys
import tempfile
import time

ROOT = Path(__file__).resolve().parents[3]
SOURCE = Path('tests/fixtures/dream-read/proof-helper-probe.c')
CONTROLS = SOURCE.with_name('private-engine-controls.c')
DRIVER = SOURCE.with_name('check-private-engine-controls.py')
FLAGS = ['-O2', '-std=c11', '-Wall', '-Wextra', '-Werror']
DISPATCH = b'return private_command_engine(context,result);'
VALIDATED = (b'if(!engine_owner_priority()||!engine_pretest_current(context)'
             b'||!engine_signature_boundary_available(context))goto refused;')
ASSERTIONS = {
    'skipped-engine': (b"admitted dispatch invokes the engine's own bounded prebirth refusal",),
    'hardcoded-refusal': (b'same functional validate/claim/15-command/semantic/settle flow',),
    'quarantine-exit': (b'same controller remains alive after answering refusal',
                        b'owner-loss observation requires same held controller alive',
                        b'owner loss is durable refusal while same controller remains alive',
                        b'independent kernel FD snapshot proves original descriptor retained after refusal',
                        b'independent observer sees original live helper still owned by this controller'),
    'quarantine-fd-release': (b'independent kernel FD snapshot proves original descriptor retained after refusal',),
    'hold-unpaid-observation': (b'interrupted prior payment forbids every new hold observation',),
    'hold-forgotten-observation-cost': (b'ordinary hold scheduling failure retains exact inclusive closed cost before another observation',),
    'hold-forgotten-final-cost': (b'final ledger scheduling failure retains its exact once-written cost and keeps custody held',),
}
LEDGERS = ('terminal-ledger.ndjson', 'live-ledger.ndjson', 'exec-ledger.ndjson',
           'unknown-ledger.ndjson', 'deadline-ledger.ndjson', 'failed-receipt-ledger.ndjson', 'hold-ledger.ndjson', 'result.json', 'claim.json')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(data):
    return (json.dumps(data, sort_keys=True, separators=(',', ':'), ensure_ascii=False) + '\n').encode()


def read_stable(path, limit):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        before = os.fstat(fd)
        if not stat.S_ISREG(before.st_mode) or before.st_size > limit:
            raise ValueError('source/evidence kind or length: ' + str(path))
        with os.fdopen(fd, 'rb', closefd=False) as source:
            data = source.read(limit + 1)
        after = os.fstat(fd)
        fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns')
        if any(getattr(before, name) != getattr(after, name) for name in fields) or len(data) != after.st_size:
            raise ValueError('source/evidence changed: ' + str(path))
        return data
    finally:
        os.close(fd)


def retain(path, data):
    with path.open('xb') as target:
        target.write(data)
    path.chmod(0o400)
    return {'path': str(path), 'len': len(data), 'sha256': sha(data)}


def run(command, directory, label, report):
    stdout_path = directory / (label + '.stdout.log')
    stderr_path = directory / (label + '.stderr.log')
    record = {'command': command, 'cwd': str(directory), 'timeout_seconds': 45,
              'synthetic_source_known': True, 'real_candidate_created': 0,
              'runtime_approved': False, 'timed_out': False, 'exit_code': None}
    report['commands'].append(record)
    began = time.monotonic_ns()
    with stdout_path.open('xb') as stdout, stderr_path.open('xb') as stderr:
        process = subprocess.Popen(command, cwd=directory, stdin=subprocess.DEVNULL,
                                   stdout=stdout, stderr=stderr, start_new_session=True)
        record['pid'] = process.pid
        try:
            record['exit_code'] = process.wait(timeout=45)
        except subprocess.TimeoutExpired:
            record['timed_out'] = True
            # Kill only the exact command PID started here. A timeout never
            # certifies descendant settlement or retries a candidate/helper.
            process.kill()
            try:
                record['exit_code'] = process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                record['exit_observation_unknown'] = True
            record['timeout_cleanup'] = 'exact command PID only; descendant settlement unproven'
    record['elapsed_ns'] = time.monotonic_ns() - began
    outputs = []
    for name, path in (('stdout', stdout_path), ('stderr', stderr_path)):
        data = read_stable(path, 16 * 1024 * 1024)
        path.chmod(0o400)
        record[name] = {'path': str(path), 'len': len(data), 'sha256': sha(data)}
        outputs.append(data)
    if record['timed_out']:
        raise ValueError('command timed out; original operation was not retried: ' + label)
    return record['exit_code'], *outputs


def expect(code, wanted, label):
    if code != wanted:
        raise ValueError(f'{label}: actual exit {code}, expected {wanted}')


def compile_source(directory, filename, report):
    executable = directory / 'program'
    code, _, _ = run(['/usr/bin/clang', *FLAGS, str(directory / filename), '-o', str(executable)],
                     directory, 'compile', report)
    expect(code, 0, filename + ' compilation')
    executable.chmod(0o500)
    report['executables'].append({'path': str(executable), 'len': executable.stat().st_size,
                                  'sha256': sha(read_stable(executable, 16 * 1024 * 1024))})
    return executable


def retain_ledgers(stdout, directory, report):
    matches = re.findall(rb'private_engine_controls=pass cases=49 synthetic_source_known=true '
                         rb'real_candidate_created=0 runtime_approved=false retained='
                         rb'(/private/tmp/dream-private-engine-controls\.[A-Za-z0-9]+)\n', stdout)
    if len(matches) != 1:
        raise ValueError('current fixture did not report the exact 49 source-known controls')
    source_dir = Path(os.fsdecode(matches[0]))
    identity = source_dir.lstat()
    if not stat.S_ISDIR(identity.st_mode) or identity.st_uid != os.geteuid() or identity.st_mode & 0o077:
        raise ValueError('source-known evidence directory custody differs')
    retained = directory / 'process-ledgers'
    retained.mkdir(mode=0o700)
    report['fixture_evidence_dir'] = str(source_dir)
    for name in LEDGERS:
        data = read_stable(source_dir / name, 2 * 1024 * 1024)
        report['process_ledgers'].append(retain(retained / name, data))


def verify(directory, inputs, report):
    source = inputs[str(SOURCE)]
    controls = inputs[str(CONTROLS)]
    hold_anchor = b'while(!owner->quarantine_settled) {'
    debt_anchor = b'if(!engine_hold_pay_prior(owner))return 0;'
    retained_anchor = (b'if(!private_book_retained_cost())return 0;\n    return 1;\n}'
                       b'\nstatic int private_pay_pending(void)')
    forgotten = (b'if(!private_book_retained_cost()){private_cost.retained_cost_ns='
                 b'private_cost.retained_raw_ns=private_cost.retained_units='
                 b'private_cost.retained_unknown_units=0;return 0;}\n    return 1;\n}'
                 b'\nstatic int private_pay_pending(void)')
    if source.count(DISPATCH) != 1 or source.count(VALIDATED) != 1 or source.count(hold_anchor) != 1 or source.count(debt_anchor) != 1 or source.count(retained_anchor) != 1:
        raise ValueError('canonical causal mutation anchors changed; no guessed mutation or retry')
    variants = {
        'current': source,
        'skipped-engine': source.replace(DISPATCH, b'(void)context;(void)result;return 0;', 1),
        'hardcoded-refusal': source.replace(
            VALIDATED, VALIDATED + b'\n    private_fail("review_unbound",'
            b'"source-known deliberately hardcoded refusal after validation");goto refused;', 1),
        'quarantine-exit': source.replace(hold_anchor, b'_exit(81);\n    while(!owner->quarantine_settled) {', 1),
        'quarantine-fd-release': source.replace(hold_anchor,
            b'if(owner->input>=0){close(owner->input);owner->input=-1;}\n    while(!owner->quarantine_settled) {', 1),
        'hold-unpaid-observation': source.replace(debt_anchor, b'(void)0;', 1),
        'hold-forgotten-observation-cost': source.replace(retained_anchor, forgotten, 1),
        'hold-forgotten-final-cost': source.replace(retained_anchor, forgotten, 1),
    }
    for name, wire in variants.items():
        own = directory / name
        own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own / SOURCE.name, wire))
        report['snapshots'].append(retain(own / CONTROLS.name, controls))
        program = compile_source(own, CONTROLS.name, report)
        arguments = [str(program)]
        if name in ('quarantine-exit', 'quarantine-fd-release'):
            arguments.append('hold-lifetime-control')
        elif name == 'hold-unpaid-observation':
            arguments.append('hold-payment-control')
        elif name == 'hold-forgotten-observation-cost':
            arguments.append('hold-scheduling-control')
        elif name == 'hold-forgotten-final-cost':
            arguments.append('hold-final-scheduling-control')
        code, stdout, stderr = run(arguments, own, 'controls', report)
        expect(code, 0 if name == 'current' else 81, name + ' controls')
        if name == 'current':
            retain_ledgers(stdout, own, report)
        elif not any(assertion in stderr for assertion in ASSERTIONS[name]):
            raise ValueError(name + ': exit81 lacks the required causal assertion')
    own = directory / 'production-no-test-hooks'
    own.mkdir(mode=0o700)
    report['snapshots'].append(retain(own / SOURCE.name, source))
    program = compile_source(own, SOURCE.name, report)
    code, stdout, _ = run([str(program), 'git', *['unused' + str(i) for i in range(8)]],
                          own, 'normal-git-refusal', report)
    expect(code, 81, 'normal git remains unqualified')
    value = json.loads(stdout)
    if value != {'profile': 'read-helper-native/1', 'availability': 'verification_unproven',
                 'whole_command_lifetime_coverage': False,
                 'basis': 'missing_supported_exact_tool_os_lifetime_capability'}:
        raise ValueError('normal-mode refusal facts differ')
    malformed = own / 'malformed.json'
    # 0600 is an intentional production-input custody requirement.
    with os.fdopen(os.open(malformed, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC, 0o600), 'wb') as target:
        target.write(b'{}')
    code, stdout, stderr = run([str(program), 'private-candidate', str(malformed), sha(b'{}')],
                               own, 'malformed-before-birth', report)
    expect(code, 81, 'malformed private input')
    if stdout or b'private candidate refused: input_invalid changed hash or noncanonical schema' not in stderr:
        raise ValueError('malformed descriptor lacks the actual strict prebirth refusal')
    malformed.chmod(0o400)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', required=True)
    args = parser.parse_args()
    target = Path(args.target_dir)
    if not target.is_absolute():
        parser.error('--target-dir must be absolute')
    target.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix='private-engine-controls-', dir=target)).resolve()
    report = {'format': 'bboxy.dream.private-engine-source-controls/1',
              'synthetic_source_known': True, 'real_candidate_created': 0,
              'runtime_approved': False, 'whole_command_lifetime_proof': False,
              'live_kernel_custody_after_probe_exit_proved': False,
              'recovery_handoff_proved': False, 'same_controller_quarantine_hold_witness_required': True, 'commands': [], 'inputs': {},
              'snapshots': [], 'executables': [], 'process_ledgers': []}
    status = 1
    try:
        inputs = {str(path): read_stable(ROOT / path, 1024 * 1024)
                  for path in (SOURCE, CONTROLS, DRIVER)}
        archive = directory / 'inputs'
        archive.mkdir(mode=0o700)
        for name, data in inputs.items():
            report['inputs'][name] = retain(archive / Path(name).name, data)
        verify(directory, inputs, report)
        if any(read_stable(ROOT / name, 1024 * 1024) != data for name, data in inputs.items()):
            raise ValueError('canonical source/control/driver changed during verification')
        report['current_sources_unchanged'] = True
        report['status'] = 'passed'
        status = 0
    except (OSError, ValueError, subprocess.SubprocessError) as error:
        report['status'] = 'failed'
        report['error'] = str(error)
    retain(directory / 'run.json', canonical(report))
    print(json.dumps({'status': report['status'], 'evidence': str(directory / 'run.json'),
                      'synthetic_source_known': True, 'real_candidate_created': 0,
                      'runtime_approved': False}, sort_keys=True))
    return status


if __name__ == '__main__':
    sys.exit(main())
