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
    'terminal-first-error-only': (b'bounded diagnostics retain later errors after first causal control failure',),
    'terminal-false-persistence': (b'actual failed persistence remains independent of causal control',),
    'terminal-skip-tail-cost': (b'actual over-CPU exit tail independently refuses after successful control and persistence',),
    'terminal-before-tail': (b'final raw verdict follows paid complete exit tail exactly once',),
    'quantum-open-slot': (b'unknown late-child slot never obtains measured basis',),
    'quantum-pre-ack': (b'counter read before positive ACK never obtains measured basis',),
    'quantum-omit-rollup': (b'actual reaped-descendant rollup is included in terminal charged conservation',),
    'quantum-raw-ticks': (b'actual Mach tick interval uses explicit retained timebase',),
    'quantum-no-wall': (b'low measured CPU cannot waive actual delayed ACK twenty-five millisecond wall gate',),
    'quantum-slot-over-wall': (b'actual second thread and per-slot CPU over wall both refuse coverage',),
    'quantum-no-conservation': (b'actual wait4 catches CPU burned after the deliberately early counter read',),
    'engine-drop-inclusive-tail': (b'actual ordinary CPU tail is retained charged paid and diagnosed before refusal',),
    'quantum-credit-controller-post': (b'already-paid controller post CPU cannot cover omitted actual helper rollup',),
    'quantum-rollup-mixed': (b'mixed second child rollup cannot certify measured basis',),
    'quantum-rollup-pre-exit': (b'rollup read before original child exit cannot acquire later proof',),
    'quantum-rollup-double': (b'live child and post-exit leader rollup cannot both be charged',),
    'quantum-rollup-forgotten-debt': (b'terminal mismatch retains once-only fallback debt even when scheduling fails',),
    'stopped-unpaid-stages': (b'stopped corroboration bounded stages pay fresh debt instead of pooling ordinary work',),
    'startup-no-terminal-ri': (b'charged never-continued startup obtains an independent paid original terminal RI before own wait4',),
    'startup-unpaid-attempt': (b'unpaid initial terminal attempt cannot release original wait4 ownership',),
    'initial-no-own-wnowait': (b'BSD ESRCH alone cannot replace original own-WNOWAIT terminal proof',),
    'teardown-unbounded': (b'fixed exit teardown ceiling rejects actual CPU burned after the last sample',),
    'teardown-uncharged-residual': (b'terminal residual without its actual retained paid debit is refused',),
    'teardown-wait4-derived': (b'fixed exit teardown ceiling rejects actual CPU burned after the last sample',),
    'teardown-unproven-child': (b'unproven child cannot mint exit-teardown process credit',),
    'teardown-quarantine-source-loss': (b'accounting quarantine revokes rebates without erasing independent child reap proof',),
    'scheduler-fixed-oversize': (b'process-free allocator must satisfy its bound and independent Q/debt assertions',),
}
PHYSICAL_SCHEDULER_DEFERRAL = {
    'status': 'deferred_unproven',
    'authority': 'Dream Read Q2 Oracle ruling2026-11-07~14:50',
    'task_etag': '9fc0d072334ccbba2331d9f66ca17acb868b540f150f01d45c0e61ebdd77faeb',
    'follow_up': 'tasks/2026-11-06-qos-builds-never-starve-mcp',
    'reason': 'physical negative stopped before causal assertion by98.854ms native creation wall; no physical oversize CPU witness',
    'retained_red': {'path': 'target/private-engine-controls-3f49ux4c/run.json',
                     'sha256': 'c50b949228b989307d236c482988272e3def1d31c52cade508bd0cb1a396759c',
                     'original_pid': 25234, 'actual_own_wait4': True, 'group_gone': True,
                     'native_cpu_ns': 337083, 'native_wall_ns': 98854000, 'debt_paid_ns': 36000000},
    'separately_callable': 'scheduler-fixed-oversize/program scheduler-control (same retained/newly compiled1.2ms source variant)',
    'physical_assertions_and_caps_unchanged': True,
    'physical_positive': 'required in canonical current baseline',
    'API_is_physical_CPU_proof': False,
    'landing_prerequisite': False,
}
LEDGERS = ('terminal-ledger.ndjson', 'live-ledger.ndjson', 'exec-ledger.ndjson',
           'unknown-ledger.ndjson', 'deadline-ledger.ndjson', 'failed-receipt-ledger.ndjson', 'hold-ledger.ndjson', 'result.json', 'claim.json', 'closure-late-stop.ndjson', 'closure-over-cpu.ndjson',
           *('diagnostic-' + str(i) + '/' + name for i in range(4) for name in ('stderr', 'stdout', 'result.json')),
           *(f'quantum-{i}.ndjson' for i in range(9)), 'charge-diagnostic.ndjson', 'stopped-stages.ndjson', 'startup-post-exit.ndjson', 'startup-unavailable.ndjson', 'startup-unpaid.ndjson', 'exit-teardown.ndjson', 'adaptive-scheduler.ndjson')


def sha(data):
    return hashlib.sha256(data).hexdigest()


def canonical(data):
    return (json.dumps(data, sort_keys=True, separators=(',', ':'), ensure_ascii=False) + '\n').encode()


def read_stable(path, limit):
    fd = os.open(path, os.O_RDONLY | os.O_NOFOLLOW | os.O_CLOEXEC)
    try:
        before = os.fstat(fd)
        if (not stat.S_ISREG(before.st_mode) or before.st_uid != os.geteuid()
                or before.st_nlink != 1 or before.st_size > limit):
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


def retain_fixture_evidence(stderr, directory, label, pid):
    matches = re.findall(
        rb'^private_engine_fixture_source_known pid=([0-9]+) retained='
        rb'(/private/tmp/dream-private-engine-controls\.[A-Za-z0-9]{6}) '
        rb'real_candidate_created=0 runtime_approved=false\n', stderr, re.MULTILINE)
    if len(matches) != 1 or int(matches[0][0]) != pid:
        raise ValueError('source-known fixture root is not bound to the exact command PID')
    source_dir = Path(os.fsdecode(matches[0][1]))
    identity = source_dir.lstat()
    if not stat.S_ISDIR(identity.st_mode) or identity.st_uid != os.geteuid() or identity.st_mode & 0o077:
        raise ValueError('source-known fixture root custody differs')
    destination = directory / (label + '-fixture-evidence')
    destination.mkdir(mode=0o700)
    record = {'source_dir': str(source_dir), 'command_pid': pid, 'root_device': identity.st_dev,
              'root_inode': identity.st_ino, 'files': [], 'not_created': [],
              'boundary': 'stable owned fixture bytes after command exit; no candidate or descendant-lifetime proof'}
    for name in LEDGERS:
        try:
            data = read_stable(source_dir / name, 2 * 1024 * 1024)
        except FileNotFoundError:
            record['not_created'].append(name)
            continue
        retained = destination / name
        retained.parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        record['files'].append(retain(retained, data))
    current = source_dir.lstat()
    if (current.st_dev, current.st_ino, current.st_uid, current.st_mode) != (
            identity.st_dev, identity.st_ino, identity.st_uid, identity.st_mode):
        raise ValueError('source-known fixture root changed during capture')
    return record


def run(command, directory, label, report, *, fixture=False):
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
    if fixture:
        # Capture before checking the verdict: a failed assertion must retain
        # the independent raw diagnostic files too, not just its final line.
        record['fixture_capture'] = retain_fixture_evidence(outputs[1], directory, label, process.pid)
    if record['timed_out']:
        raise ValueError('command timed out; original operation was not retried: ' + label)
    return record['exit_code'], *outputs


def expect(code, wanted, label):
    if code != wanted:
        raise ValueError(f'{label}: actual exit {code}, expected {wanted}')


def verify_lifetime_mutant_cleanup(stderr, record):
    # The deliberately broken controller cannot prove product custody. This
    # existing fixture observer must nevertheless discharge its OWN child PID
    # and release only its original helper gate, with independently observed
    # original-incarnation terminal/group facts. It never adopts a child wait4.
    rows = re.findall(
        rb'^private_hold_fixture_cleanup_source_known controller=([0-9]+) '
        rb'controller_reaped=1 controller_status_known=1 controller_status_raw=([0-9]+) '
        rb'retained_controller_pid=0 helper=([0-9]+) helper_birth=([0-9]+) helper_start=([0-9]+) '
        rb'helper_identity_known=1 original_helper_terminal_observed=1 original_group_gone=1 '
        rb'release_gate_closed=1 signals_sent=0 helper_reaped_by_observer=false deadline_exhausted=0 '
        rb'debt_charged_ns=([0-9]+) debt_paid_ns=([0-9]+) runtime_approved=false\n', stderr, re.MULTILINE)
    if len(rows) != 1:
        raise ValueError('lifetime mutant lacks bounded original-identity cleanup evidence')
    controller, status, helper, birth, start, charged, paid = map(int, rows[0])
    if (status != 81 << 8 or min(controller, helper, birth, start) <= 0
            or len({record['pid'], controller, helper}) != 3 or charged != paid):
        raise ValueError('lifetime mutant cleanup status, identity or paid debt differs')
    record['fixture_failure_cleanup_observed'] = {
        'controller': controller, 'controller_actual_reaped': True, 'controller_status_raw': status,
        'retained_controller_pid': 0, 'helper': helper, 'helper_birth': birth, 'helper_start': start,
        'helper_terminal_observed': True, 'original_group_gone': True, 'signals_sent': 0,
        'helper_reaped_by_observer': False, 'debt_charged_ns': charged, 'debt_paid_ns': paid,
        'boundary': 'fixture gate EOF and original identity observations; broken controller custody remains failed'}


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
    matches = re.findall(rb'private_engine_controls=pass cases=72 synthetic_source_known=true '
                         rb'real_candidate_created=0 runtime_approved=false retained='
                         rb'(/private/tmp/dream-private-engine-controls\.[A-Za-z0-9]+)\n', stdout)
    if len(matches) != 1:
        raise ValueError('current fixture did not report the exact 72 source-known controls')
    source_dir = Path(os.fsdecode(matches[0]))
    if str(source_dir) != report['commands'][-1]['fixture_capture']['source_dir']:
        raise ValueError('completed fixture root differs from its original command binding')
    identity = source_dir.lstat()
    if not stat.S_ISDIR(identity.st_mode) or identity.st_uid != os.geteuid() or identity.st_mode & 0o077:
        raise ValueError('source-known evidence directory custody differs')
    retained = directory / 'process-ledgers'
    retained.mkdir(mode=0o700)
    report['fixture_evidence_dir'] = str(source_dir)
    for name in LEDGERS:
        data = read_stable(source_dir / name, 2 * 1024 * 1024)
        (retained / name).parent.mkdir(mode=0o700, parents=True, exist_ok=True)
        report['process_ledgers'].append(retain(retained / name, data))


def verify_setup_budget(program, directory, report):
    code, stdout, _ = run([str(program), 'diagnostic-setup-budget-control'],
                          directory, 'setup-budget', report, fixture=True)
    expect(code, 0, 'diagnostic setup measured-overrun control')
    rows = re.findall(
        rb'^private_diagnostic_setup_budget_source_known guard_return=0 clocks_known=1 '
        rb'observed_cpu_ns=([0-9]+) raw_cpu_ns=([0-9]+) debt_charged_ns=([0-9]+) '
        rb'debt_paid_ns=([0-9]+) full_cost_retained=true control_only=true '
        rb'real_candidate_created=0 runtime_approved=false whole_command_lifetime_proof=false\n',
        stdout, re.MULTILINE)
    if len(rows) != 1:
        raise ValueError('diagnostic setup lacks its exact measured-overrun evidence')
    observed, raw, charged, paid = map(int, rows[0])
    if not (2_000_000 <= observed <= raw <= (2**64 - 1) // 9 and charged == paid == 9 * raw):
        raise ValueError('diagnostic setup overrun cost or paid debt differs')
    report['commands'][-1]['closed_setup_cost_checked'] = {
        'observed_cpu_ns': observed, 'raw_cpu_ns': raw, 'debt_charged_ns': charged,
        'debt_paid_ns': paid, 'boundary': 'one closed source-known setup unit; not whole-command proof'}


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
    error_anchor = b'private_record_error(code,detail);'
    persistence_anchor = (b'private_terminal.persist_ok=private_terminal.persist_open&&private_terminal.persist_write&&private_terminal.persist_fsync\n'
                          b'        &&private_terminal.persist_close&&private_terminal.persist_dir_fsync;')
    cost_anchor = b'private_terminal.cost_ok=0;'
    tail_start = source.index(b'    fprintf(stderr,"private_candidate_exit_tail ')
    tail_end = source.index(b'private_terminal.tail_complete=1;', tail_start) + len(b'private_terminal.tail_complete=1;')
    final_end = source.index(b'\n}\nstatic int private_early_refusal', tail_end)
    if any(source.count(anchor) != 1 for anchor in (error_anchor, persistence_anchor, cost_anchor)):
        raise ValueError('terminal causal mutation anchors changed')
    early_terminal = source[:tail_start] + source[tail_end:final_end] + source[tail_start:tail_end] + source[final_end:]
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
        'terminal-first-error-only': source.replace(error_anchor, b'if(!private_failed)private_record_error(code,detail);', 1),
        'terminal-false-persistence': source.replace(persistence_anchor, b'private_terminal.persist_ok=1;', 1),
        'terminal-skip-tail-cost': source.replace(cost_anchor, b'(void)ok;', 1),
        'terminal-before-tail': early_terminal,
    }
    quantum_anchors = {
        'unknown': b'g->unknown_mask==0&&g->baseline_mask==all',
        'post_ack': b'&&((g->post_ack_mask|g->wait4_mask|g->rollup_mask)&all)==all&&g->timebase_numer&&g->timebase_denom;',
        'read_time': b'g->counter_read[i]<g->end[i]',
        'rollup': b'b->state.raw_rusage_cpu>a->state.raw_waited?b->state.raw_rusage_cpu:a->state.raw_waited',
        'conversion': b'return engine_scaled_ticks(raw,scale.numer,scale.denom,1,ns);',
        'wall': b'g->charge<=1000000&&g->windows<=25000000',
        'slot': b'if((g->counter_mask&(1u<<i))&&g->slot_cpu[i]>wall)return LEDGER_COVERAGE;',
        'conservation': b'if(owner->conservation_excess)return engine_conservation_failure(owner,"resource_limit","actual terminal CPU exceeds independently charged helper conservation bound");',
        'tail': b'else if(inclusive>actual) {',
        'controller_credit': b'||!engine_safe_sum(credit,owner->justified_post_cpu,&credit)',
        'rollup_unique': b'||!g->rollup_unique||!g->rollup_child_retired',
        'rollup_before_exit': b'||g->rollup_read_ns<g->rollup_child_exit_ns||g->rollup_read_ns<g->rollup_child_zero_ns',
        'rollup_double': b'if(g->live_child_cpu_ns||g->rollup_child_cpu_ns!=g->slot_cpu[1])return LEDGER_COVERAGE;',
        'rollup_debt': b'if(difference>UINT64_MAX/9||!private_retain_debt_adjustment(difference))return 0;',
        'stopped_payment': b'int paid=engine_charge_at(owner,used,used,1,stage,__LINE__);',
        'startup_read': b'int read=engine_initial_ri_read(owner->ticket.pid,&usage);',
        'initial_attempt_paid': b'owner->post_exit_attempt_paid=(unsigned)paid;return paid;',
        'initial_own_wnowait': b'||!p->own_wnowait_known||!p->own_wnowait_terminal',
        'teardown_limit': b'owner->exit_teardown_limit_ns=ENGINE_EXIT_TEARDOWN_CEILING_NS;',
        'residual_paid': b'owner->residual_cost_paid=(unsigned)paid;return paid;',
        'teardown_count': b'return *direct+*child;',
        'quarantine_provisional': b'if(owner->provisional_count&&!owner->provisional_finalized)(void)engine_revoke_provisional(owner);',
        'scheduler_window': b'return owner->scheduler.used_run;',
    }
    if any(source.count(anchor) != 1 for anchor in quantum_anchors.values()):
        raise ValueError('66236 causal mutation anchor differs; no guessed mutation')
    variants.update({
        'quantum-open-slot': source.replace(quantum_anchors['unknown'], b'1&&g->baseline_mask==all', 1),
        'quantum-pre-ack': source.replace(quantum_anchors['post_ack'], b'&&g->timebase_numer&&g->timebase_denom;', 1).replace(
            quantum_anchors['read_time'], b'0', 1),
        'quantum-omit-rollup': source.replace(quantum_anchors['rollup'], b'b->state.raw_rusage_cpu', 1),
        'quantum-raw-ticks': source.replace(quantum_anchors['conversion'], b'*ns=raw;return 1;', 1),
        'quantum-no-wall': source.replace(quantum_anchors['wall'], b'g->charge<=1000000', 1),
        'quantum-slot-over-wall': source.replace(quantum_anchors['slot'], b'(void)wall;', 1),
        'quantum-no-conservation': source.replace(quantum_anchors['conservation'], b'(void)owner->conservation_excess;', 1),
        'engine-drop-inclusive-tail': source.replace(quantum_anchors['tail'], b'else if(0&&inclusive>actual) {', 1),
        'quantum-credit-controller-post': source.replace(quantum_anchors['controller_credit'],
            quantum_anchors['controller_credit'] + b'\n        ||!engine_safe_sum(credit,owner->result->post_cpu_ns.observed?owner->result->post_cpu_ns.value:0,&credit)', 1),
        'quantum-rollup-mixed': source.replace(quantum_anchors['rollup_unique'], b'||!g->rollup_child_retired', 1),
        'quantum-rollup-pre-exit': source.replace(quantum_anchors['rollup_before_exit'], b'||0', 1),
        'quantum-rollup-double': source.replace(quantum_anchors['rollup_double'], b'(void)g->live_child_cpu_ns;', 1),
        'quantum-rollup-forgotten-debt': source.replace(quantum_anchors['rollup_debt'], b'(void)private_retain_debt_adjustment;if(difference>UINT64_MAX/9)return 0;', 1),
        'stopped-unpaid-stages': source.replace(quantum_anchors['stopped_payment'], b'(void)used;(void)stage;int paid=1;', 1),
        'startup-no-terminal-ri': source.replace(quantum_anchors['startup_read'], b'(void)engine_initial_ri_read;int read=-1;errno=ENOTSUP;', 1),
        'startup-unpaid-attempt': source.replace(quantum_anchors['initial_attempt_paid'], b'(void)paid;owner->post_exit_attempt_paid=owner->post_exit_initial_source_valid&&owner->post_exit_attempt_retained;return owner->post_exit_attempt_paid;', 1),
        'initial-no-own-wnowait': source.replace(quantum_anchors['initial_own_wnowait'], b'||0', 1),
        'teardown-unbounded': source.replace(quantum_anchors['teardown_limit'], b'owner->exit_teardown_limit_ns=30000000000ULL;', 1),
        'teardown-uncharged-residual': source.replace(quantum_anchors['residual_paid'], b'(void)paid;owner->residual_cost_paid=1;return 1;', 1),
        'teardown-wait4-derived': source.replace(quantum_anchors['teardown_limit'], b'owner->exit_teardown_limit_ns=owner->wait_user+owner->wait_system+2000;', 1),
        'teardown-unproven-child': source.replace(quantum_anchors['teardown_count'], b'return *direct+(owner->slots==2);', 1),
        'teardown-quarantine-source-loss': source.replace(quantum_anchors['quarantine_provisional'],
            b'owner->rollup_source_failed=1;' + quantum_anchors['quarantine_provisional'], 1),
        'scheduler-fixed-oversize': source.replace(quantum_anchors['scheduler_window'], b'(void)owner;return 1200000ULL;', 1),
    })
    quantum_modes = {'quantum-open-slot': 0, 'quantum-pre-ack': 1, 'quantum-omit-rollup': 2,
                     'quantum-raw-ticks': 3, 'quantum-no-wall': 4, 'quantum-slot-over-wall': 5,
                     'quantum-no-conservation': 1, 'quantum-credit-controller-post': 2,
                     'quantum-rollup-mixed': 6, 'quantum-rollup-pre-exit': 7,
                     'quantum-rollup-double': 8, 'quantum-rollup-forgotten-debt': 8,
                     'teardown-unproven-child': 2, 'teardown-quarantine-source-loss': 2}
    for name, wire in variants.items():
        own = directory / name
        own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own / SOURCE.name, wire))
        report['snapshots'].append(retain(own / CONTROLS.name, controls))
        program = compile_source(own, CONTROLS.name, report)
        arguments = [str(program)]
        if name.startswith('terminal-'):
            arguments.append('terminal-diagnostic-controls')
        elif name in ('quarantine-exit', 'quarantine-fd-release'):
            arguments.append('hold-lifetime-control')
        elif name == 'hold-unpaid-observation':
            arguments.append('hold-payment-control')
        elif name == 'hold-forgotten-observation-cost':
            arguments.append('hold-scheduling-control')
        elif name == 'hold-forgotten-final-cost':
            arguments.append('hold-final-scheduling-control')
        elif name in quantum_modes:
            arguments.extend(['quantum-control', str(quantum_modes[name])])
        elif name == 'engine-drop-inclusive-tail':
            arguments.append('charge-diagnostic-control')
        elif name in ('stopped-unpaid-stages', 'startup-no-terminal-ri', 'startup-unpaid-attempt', 'initial-no-own-wnowait'):
            mode = {'stopped-unpaid-stages': '0', 'startup-no-terminal-ri': '1', 'startup-unpaid-attempt': '3', 'initial-no-own-wnowait': '0'}[name]
            arguments.extend(['initial-stopped-control', mode])
        elif name in ('teardown-unbounded', 'teardown-uncharged-residual', 'teardown-wait4-derived'):
            arguments.extend(['exit-teardown-control', '1' if name == 'teardown-uncharged-residual' else '0'])
        elif name == 'scheduler-fixed-oversize':
            arguments.append('scheduler-api-control')
        code, stdout, stderr = run(arguments, own, 'controls', report, fixture=name != 'scheduler-fixed-oversize')
        expect(code, 0 if name == 'current' else 81, name + ' controls')
        if name == 'current':
            retain_ledgers(stdout, own, report)
            verify_setup_budget(program, own, report)
        elif not any(assertion in stderr for assertion in ASSERTIONS[name]):
            raise ValueError(name + ': exit81 lacks the required causal assertion')
        if name == 'scheduler-fixed-oversize':
            prefix = (b'private_scheduler_allocator_api slots=1 run_ns=1200000 cap_ns=350000 '
                      b'expected_initial_ns=350000 aggregate_target_ns=550000 supplied_A_ns=350000 '
                      b'supplied_C_ns=250000 supplied_W_ns=1400000 computed_Q_ns=600000 '
                      b'ledger_verdict=0 projected_debt_ns=9000000')
            if (prefix not in stderr or stdout or b'fixture_native_birth' in stderr
                    or b'actual_helper_quantum_witness=false' not in stderr
                    or b'CPU_from_run_or_wall=false native_births=0 helper_CPU_observed=false' not in stderr):
                raise ValueError('canonical allocator API mutant lacks its separately labelled process-free causal receipt')
            report['commands'][-1]['causal_scope'] = 'real_c_allocator_API_bound_with_independent_supplied_Q_debt_inputs'
            report['physical_scheduler_negative'] = PHYSICAL_SCHEDULER_DEFERRAL
        if name in ('quarantine-exit', 'quarantine-fd-release'):
            verify_lifetime_mutant_cleanup(stderr, report['commands'][-1])
    anchor = b'int ok=private_end_unit(step->start);'
    if controls.count(anchor) != 1:
        raise ValueError('diagnostic setup cost mutation anchor differs')
    own = directory / 'fixture-forgotten-setup-cost'
    own.mkdir(mode=0o700)
    report['snapshots'].append(retain(own / SOURCE.name, source))
    mutant = controls.replace(anchor, b'private_cost.open_cpu=clock_ns(CLOCK_THREAD_CPUTIME_ID);' + anchor, 1)
    report['snapshots'].append(retain(own / CONTROLS.name, mutant))
    program = compile_source(own, CONTROLS.name, report)
    code, _, stderr = run([str(program), 'diagnostic-setup-budget-control'],
                           own, 'controls', report, fixture=True)
    expect(code, 81, 'discarded diagnostic setup cost control')
    if b'diagnostic setup overrun retains full paid cost before refusal' not in stderr:
        raise ValueError('discarded setup cost lacks its own causal assertion')
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


def verify_scheduler_api(directory, inputs, report):
    """Supplemental API receipt, distinct from the deferred physical negative."""
    source = inputs[str(SOURCE)]
    controls = inputs[str(CONTROLS)]
    anchor = b'return owner->scheduler.used_run;'
    feedback_anchor = b'uint64_t next=p->used_run,target=engine_scheduler_target(p);'
    shrink_anchor = b'if(g->actual_cpu!=UINT64_MAX&&g->actual_cpu>target) {'
    assertion = b'process-free allocator must satisfy its bound and independent Q/debt assertions'
    feedback_assertion = b'process-free feedback compares CPU with newly reduced target while preserving original allocation'
    shrink_assertion = b'process-free finite observed CPU shrinks window basis without earning proof or unknown-controller growth'
    if source.count(anchor) != 1 or source.count(feedback_anchor) != 1 or source.count(shrink_anchor) != 1 or controls.count(b'static void scheduler_api_control(void) {') != 1:
        raise ValueError('supplemental scheduler API anchors changed')
    variants = {'api-current': source,
                'api-fixed-oversize': source.replace(anchor, b'(void)owner;return 1200000ULL;', 1),
                'api-old-target-feedback': source.replace(feedback_anchor, b'uint64_t next=p->used_run,target=p->used_target;', 1),
                'api-skip-window-shrink': source.replace(shrink_anchor, b'if(g->measured&&controller!=UINT64_MAX&&g->actual_cpu!=UINT64_MAX&&g->actual_cpu>target) {', 1)}
    for name, wire in variants.items():
        own = directory / name
        own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own / SOURCE.name, wire))
        report['snapshots'].append(retain(own / CONTROLS.name, controls))
        program = compile_source(own, CONTROLS.name, report)
        code, stdout, stderr = run([str(program), 'scheduler-api-control'], own, 'scheduler-api', report)
        expect(code, 0 if name == 'api-current' else 81, name + ' supplemental allocator API')
        if stdout or b'private_engine_fixture_source_known' in stderr or b'fixture_native_birth' in stderr:
            raise ValueError('process-free API emitted a physical fixture/birth receipt')
        rows = re.findall(rb'^private_scheduler_allocator_api (.+)$', stderr, re.MULTILINE)
        fields = [dict(re.findall(rb'([A-Za-z_][A-Za-z_0-9]*)=([^ ]+)', row)) for row in rows]
        if not fields or any(f.get(b'native_births') != b'0'
                             or f.get(b'helper_CPU_observed') != b'false'
                             or f.get(b'actual_helper_quantum_witness') != b'false'
                             or f.get(b'CPU_from_run_or_wall') != b'false' for f in fields):
            raise ValueError('API evidence lacks its explicit finite-counter/no-helper boundary')
        if name == 'api-current':
            if (len(fields) != 2 or [f.get(b'slots') for f in fields] != [b'1', b'2']
                    or [f.get(b'run_ns') for f in fields] != [b'350000', b'175000']
                    or any(f.get(b'computed_Q_ns') != b'600000' or f.get(b'projected_debt_ns') != b'9000000' for f in fields)
                    or b'private_scheduler_api_complete typed_slot_cases=2 ' not in stderr
                    or b'private_scheduler_api_crossing old_target_ns=550000 measured_A_ns=500000 observed_C_ns=350000 new_target_ns=450000 previous_run_ns=350000 next_run_ns=175000 decrease_count=1 old_allocation_unchanged=true' not in stderr
                    or b'private_scheduler_api_window_shrink measured_basis=false observed_A_ns=600000 observed_C_ns=250000 controller_known=true next_target_ns=550000 previous_run_ns=350000 next_run_ns=175000 decrease_count=1 growth_count=0' not in stderr
                    or b'private_scheduler_api_window_shrink measured_basis=false observed_A_ns=600000 observed_C_ns=18446744073709551615 controller_known=false next_target_ns=550000 previous_run_ns=350000 next_run_ns=175000 decrease_count=1 growth_count=0' not in stderr
                    or b'private_scheduler_api_max_latency live_ACK_delays_ns=100000,300000 selected_latency_ns=300000 reserved_slots=2 next_run_ns=25000' not in stderr
                    or b'supplied_over_CPU_ns=1200000 computed_over_Q_ns=1450000 projected_over_debt_ns=13050000' not in stderr
                    or b'physical_fixed_oversize_witness=deferred_unproven canonical_suite_receipt=false' not in stderr):
                raise ValueError('current API lacks complete reserved-slot/feedback/Q/debt checks')
        elif name == 'api-fixed-oversize':
            failures = re.findall(rb'^probe refused: (.+?) errno=-?[0-9]+$', stderr, re.MULTILINE)
            if assertion not in failures or len(fields) != 1 or fields[0].get(b'run_ns') != b'1200000':
                raise ValueError('API oversize mutation lacks its exact allocator-bound causal failure')
            report['api_mutant_failed_axis'] = 'allocator bound; supplied independent CPU Q/debt inputs stay unchanged'
        elif name == 'api-old-target-feedback':
            failures = re.findall(rb'^probe refused: (.+?) errno=-?[0-9]+$', stderr, re.MULTILINE)
            crossing = b'private_scheduler_api_crossing old_target_ns=550000 measured_A_ns=500000 observed_C_ns=350000 new_target_ns=450000 previous_run_ns=350000 next_run_ns=350000 decrease_count=0 old_allocation_unchanged=true'
            if feedback_assertion not in failures or len(fields) != 2 or crossing not in stderr:
                raise ValueError('old-target API mutation lacks its exact known-reserve crossing failure')
            report['api_feedback_mutant_failed_axis'] = 'old target fails to halve actual previous run after new controller reserve'
        else:
            failures = re.findall(rb'^probe refused: (.+?) errno=-?[0-9]+$', stderr, re.MULTILINE)
            shrink = b'private_scheduler_api_window_shrink measured_basis=false observed_A_ns=600000 observed_C_ns=250000 controller_known=true next_target_ns=550000 previous_run_ns=350000 next_run_ns=350000 decrease_count=0 growth_count=0'
            if shrink_assertion not in failures or len(fields) != 2 or shrink not in stderr:
                raise ValueError('unmeasured-basis API mutation lacks its exact finite-burn decrease failure')
            report['api_window_mutant_failed_axis'] = 'finite observed burn must shrink even without measured-basis certification'
        report['commands'][-1]['process_free_allocator_api'] = True
    report['canonical_suite_receipt'] = False
    report['physical_fixed_oversize_witness'] = 'deferred_unproven under Oracle14:50 ruling; no API claim of physical CPU'
    report['physical_scheduler_negative'] = PHYSICAL_SCHEDULER_DEFERRAL


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', required=True)
    parser.add_argument('--scheduler-api-only', action='store_true',
                        help='supplemental process-free allocator API checks; not a canonical physical-suite receipt')
    args = parser.parse_args()
    target = Path(args.target_dir)
    if not target.is_absolute():
        parser.error('--target-dir must be absolute')
    target.mkdir(parents=True, exist_ok=True)
    directory = Path(tempfile.mkdtemp(prefix='private-engine-scheduler-api-' if args.scheduler_api_only else 'private-engine-controls-', dir=target)).resolve()
    report = {'format': 'bboxy.dream.private-engine-scheduler-api-controls/1' if args.scheduler_api_only else 'bboxy.dream.private-engine-source-controls/1',
              'synthetic_source_known': True, 'real_candidate_created': 0,
              'runtime_approved': False, 'whole_command_lifetime_proof': False,
              'live_kernel_custody_after_probe_exit_proved': False,
              'recovery_handoff_proved': False, 'same_controller_quarantine_hold_witness_required': True, 'commands': [], 'inputs': {},
              'snapshots': [], 'executables': [], 'process_ledgers': [],
              'physical_scheduler_negative': PHYSICAL_SCHEDULER_DEFERRAL}
    status = 1
    try:
        inputs = {str(path): read_stable(ROOT / path, 1024 * 1024)
                  for path in (SOURCE, CONTROLS, DRIVER)}
        archive = directory / 'inputs'
        archive.mkdir(mode=0o700)
        for name, data in inputs.items():
            report['inputs'][name] = retain(archive / Path(name).name, data)
        if args.scheduler_api_only:
            verify_scheduler_api(directory, inputs, report)
        else:
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
    summary = {'status': report['status'], 'evidence': str(directory / 'run.json'),
               'synthetic_source_known': True, 'real_candidate_created': 0, 'runtime_approved': False}
    if args.scheduler_api_only:
        summary['scope'] = 'supplemental_process_free_scheduler_api'
    print(json.dumps(summary, sort_keys=True))
    return status


if __name__ == '__main__':
    sys.exit(main())
