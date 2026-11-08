# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Run canonical synthetic private-engine controls; never run a real candidate."""
import argparse
import errno
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

# Eight decision66280 controls plus independent preserved own-thread, semantic,
# persistence and original-controller custody controls. No retired graph/control
# is re-labelled as a current pass.
LEDGERS = ('hold-ledger.ndjson', 'hold-control-success', 'claim.json', 'charge-diagnostic.ndjson',
           'admitted-descriptor', 'retained-descriptor', 'currentness-a', 'currentness-b', 'deadline-result/stdout', 'deadline-result/result.json',
           *(f'total-{i}.ndjson' for i in range(27)), 'knote-rollup-26', *(f'live-race-{i}' for i in range(16,23)), *(f'event-release-{i}' for i in range(23,26)), 'rollup-2', 'rollup-2-terminal', 'rollup-2-release', 'release-6',
           *(f'pause-{i}-{phase}' for i in (9,10) for phase in ('start','done','release')),
           *('diagnostic-' + str(i) + '/' + name for i in range(4) for name in ('stderr', 'stdout', 'result.json')))
ASSERTIONS = {
 'census-clear-new-delivery': b'observing census consumes only old coalesced wake and preserves every nested new delivery',
 'sample-old-combined-unit': b'retained-child sample composition closes every full-cost phase before the one millisecond hard bound',
 'batch-dropped-currentness-check': b'batched currentness still refuses an actual retained-file change with every full cost paid',
 'leader-refresh-use-cached': b'cached terminal leader RI predating child receipt cannot satisfy fresh rollup observation',
 'child-exit-ignore-note': b'retained process-bound NOTE_EXIT qualifies RI_ESRCH with unchanged rollup coverage across approved exec',
 'child-exit-unbound-note': b'unbound or incomplete child exit witness retains rollup refusal and sticky child-cost state',
 'verifier-pre-registration-run': b'actual leader registration and owned close-on-exec queue precede first controlled continuation',
 'verifier-event-as-proof': b'fast scripted between-poll fork notification produces distinct honest missed-verifier refusal',
 'verifier-skip-terminal-drain': b'actual queued leader fork survives child reap and terminal precheck before census',
 'verifier-poll-only': b'actual queued leader fork survives child reap and terminal precheck before census',
 'verifier-ignore-registration': b'failed leader event registration refuses coverage before any controlled continuation and still settles original effect',
 'verifier-early-queue-close': b'owned event queue cannot close before original physical settlement',
 'live-sample-closing-no-rescue': b'closing TASKALLINFO ESRCH reuses the same qualified original terminal rescue',
 'live-sample-errno-as-exit': b'ESRCH while exact original remains live never substitutes errno for terminal proof',
 'live-sample-no-recheck': b'ESRCH after injected original exit rechecks qualified terminal proof and settles successfully',
 'live-sample-ignore-owned-row': b'ESRCH with supplied foreign birth or full token mismatch remains coverage unknown',
 'live-sample-any-error': b'nonESRCH read failure never uses otherwise available terminal proof',
 'process-create-old25': b'34ms process-create policy admits and records exact9ms excess on actual owned fixture',
 'process-create-ignore-cpu': b'process-create CPU above2ms refuses without CONT after actual owned settlement',
 'argv-unknown-brace-accepted': b'unregistered or malformed brace syntax refuses instead of copying unknown literals',
 'argv-tree-literal-reject': b'actual registry tree_pin argv preparation preserves selected commit and literal tree selector',
 'helper-cpu-not-wall': b'long wall helper adds only nine times wait4 CPU above its existing global debt baseline',
 'total-helper-floor': b'helper CPU debt API preserves one millisecond floor and conservative unknown cost',
 'currentness-empty-floors': b'same bounded currentness open stat and close share one fully paid unit',
 'terminal-descriptor-hash': b'admitted FD fresh hash refuses actual in-place mutation despite identical length',
 'terminal-work-deadline': b'expired original work deadline forbids next work after full cost payment',
 'terminal-deadline-output': b'expired deadline still attempts one bounded fully paid refusal result and persistence',
 'path-component-floor': b'one bounded path walk pays actual work without a floor per tiny component',
 'total-qos': b'background atomic spawn and unchanged native/application gates before first pipe',
 'total-live-only': b'actual terminal wait4 refuses CPU burned beyond last live sample',
 'total-drop-rollup': b'authoritative leader wait4 retains actual reaped-child CPU rollup',
 'total-no-live-kill': b'live cap crossing sends original group kill before terminal verdict',
 'total-skip-helper-debt': b'actual terminal helper debt precedes next expensive work',
 'total-no-foreground-stop': b'foreground one STOP positive ACK no user growth and admitted single resume',
 'total-false-eperm-settlement': b'EPERM with live original retains same-owner quarantine until actual ownreap and groupgone',
 'total-pause-terminal-user': b'terminal actual below-cap user growth after ACK refuses without any product resume',
 'total-pause-live-user': b'live actual below-cap user growth after ACK remains sticky through ownwait4',
 'total-unknown-rss': b'unknown terminal RSS stays unavailable while full known wait4 CPU debt is paid',
 'total-negative-rss-as-zero': b'negative terminal RSS API remains unknown rather than zero or a finite breach',
 'total-terminal-rss': b'actual terminal wait4 RSS above16MiB refuses after ownreap and full CPU payment',
 'total-sample-gap': b'known live and paused RI start gaps over50ms refuse without resetting prior evidence',
 'total-rest-slice': b'absolute-debt rest loop requests at most one millisecond per sleep',
 'total-ri-return-gap': b'RI-return API retains old begin through forty plus twenty milliseconds',
 'total-terminal-leader-skip-child': b'terminal leader API cannot skip required retained-member group kill',
 'total-escaped-token-cleanup': b'escaped cleanup API requires current exact-birth outside-group recipient',
 'total-child-unproven': b'unproven child API refuses without relabelling or erasing known leader CPU debt',
 'total-child-rollup-disappearance': b'child disappearance API requires qualified nonduplicated terminal-counter rollup',
 'total-native-false-empty': b'native census API zero with EIO is unknown rather than an empty child or group',
 'total-stale-settlement-unknown': b'natural-exit pending final kill resolves errno after ownreap groupgone without stale unknown',
 'skipped-engine': b"admitted dispatch invokes the engine's own bounded prebirth refusal",
 'hardcoded-refusal': b'same functional validate/claim/15-command/semantic/settle flow',
 'quarantine-exit': b'same controller remains alive until external fixture release',
 'quarantine-fd-release': b'independent kernel FD snapshot proves original descriptor retained after refusal',
 'hold-unpaid-observation': b'interrupted prior payment forbids every new hold observation',
 'terminal-first-error-only': b'bounded diagnostics retain later errors after first causal control failure',
 'terminal-false-persistence': b'actual failed persistence remains independent of causal control',
 'terminal-skip-tail-cost': b'actual over-CPU exit tail independently refuses after successful control and persistence',
 'terminal-before-tail': b'final raw verdict follows paid complete exit tail exactly once',
 'engine-drop-inclusive-tail': b'actual ordinary CPU tail is retained charged paid and diagnosed before refusal',
 'ordinary-before-debt': b'diagnostic setup overrun retains full paid cost before refusal',
}

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
        rb'helper_identity_known=1 original_helper_terminal_observed=1 original_group_gone=1 original_group_errno=0 '
        rb'release_gate_closed=1 release_path=(normal_byte_close|failure_eof_close) '
        rb'release_write_result=(-?[0-9]+) release_write_errno=([0-9]+) '
        rb'release_close_result=0 release_close_errno=0 signals_sent=0 helper_reaped_by_observer=false deadline_exhausted=0 '
        rb'debt_charged_ns=([0-9]+) debt_paid_ns=([0-9]+) runtime_approved=false\n', stderr, re.MULTILINE)
    if len(rows) != 1:
        raise ValueError('lifetime mutant lacks bounded original-identity cleanup evidence')
    controller, status, helper, birth, start, release_path, write_result, write_error, charged, paid = rows[0]
    controller, status, helper, birth, start, write_result, write_error, charged, paid = map(int,
        (controller, status, helper, birth, start, write_result, write_error, charged, paid))
    if (status != 81 << 8 or min(controller, helper, birth, start) <= 0
            or len({record['pid'], controller, helper}) != 3 or charged != paid):
        raise ValueError('lifetime mutant cleanup status, identity or paid debt differs')
    if write_error or write_result != (1 if release_path == b'normal_byte_close' else -2):
        raise ValueError('lifetime mutant release lacks exact successful original byte+close or no-write EOF+close provenance')
    record['fixture_failure_cleanup_observed'] = {
        'controller': controller, 'controller_actual_reaped': True, 'controller_status_raw': status,
        'retained_controller_pid': 0, 'helper': helper, 'helper_birth': birth, 'helper_start': start,
        'helper_terminal_observed': True, 'original_group_gone': True, 'signals_sent': 0,
        'helper_reaped_by_observer': False, 'debt_charged_ns': charged, 'debt_paid_ns': paid,
        'original_release': {'path': release_path.decode(), 'write_result': write_result, 'write_errno': write_error,
                             'actual_close_result': 0, 'close_errno': 0, 'known_closed': True},
        'boundary': 'exact original fixture byte-or-EOF release plus identity observations; broken controller custody remains failed'}


def verify_controller_liveness_mutant(stderr, record):
    # A common invariant covers every pre-release liveness observation. Accept
    # only the actual original child's positive consuming wait, and the exact
    # injected exit branch; unknown wait errors or another child assertion do
    # not satisfy this deliberately broken custody control.
    rows = re.findall(
        rb'^private_hold_controller_liveness_source_known '
        rb'stage=(post_refusal|owner_loss_pending|owner_loss_recorded|before_external_release) '
        rb'controller=([0-9]+) wait_result=([0-9]+) wait_errno=0 '
        rb'controller_reaped=1 controller_status_known=1 controller_status_raw=([0-9]+) '
        rb'retained_controller_pid=0 release_gate_held=1 runtime_approved=false\n', stderr, re.MULTILINE)
    exits = re.findall(
        rb'^private_hold_causal_exit_source_known controller=([0-9]+) helper=([0-9]+) '
        rb'quarantined=1 refusal_attempted=1 refusal_complete=1 own_wait4=0 group_gone=0 '
        rb'runtime_approved=false\n', stderr, re.MULTILINE)
    if len(rows) != 1 or len(exits) != 1:
        raise ValueError('early-exit mutant lacks exact pre-release original-controller liveness and causal-branch evidence')
    stage, controller, joined, status = rows[0]
    controller, joined, status = map(int, (controller, joined, status))
    exit_controller, helper = map(int, exits[0])
    cleanup = record['fixture_failure_cleanup_observed']
    if (controller != joined or controller != exit_controller or status != 81 << 8
            or controller != cleanup['controller'] or helper != cleanup['helper']):
        raise ValueError('early-exit liveness evidence differs from the exact causal controller/helper cleanup')
    record['pre_release_controller_liveness_failure'] = {
        'stage': stage.decode(), 'controller': controller, 'actual_consuming_wait_pid': joined,
        'raw_status': status, 'release_gate_held': True, 'exact_mutant_exit_branch_observed': True}


def compile_source(directory, filename, report):
    executable = directory / 'program'
    code, _, _ = run(['/usr/bin/clang', *FLAGS, str(directory / filename), '-o', str(executable)],
                     directory, 'compile', report)
    expect(code, 0, filename + ' compilation')
    executable.chmod(0o500)
    report['executables'].append({'path': str(executable), 'len': executable.stat().st_size,
                                  'sha256': sha(read_stable(executable, 16 * 1024 * 1024))})
    return executable



def changed(source, anchor, replacement):
    if source.count(anchor) != 1:
        raise ValueError('causal mutation anchor differs: ' + repr(anchor))
    return source.replace(anchor, replacement, 1)


def variants(source):
    result = {}
    def add(name, anchor, replacement, args):
        result[name] = (changed(source, anchor, replacement), args)
    add('census-clear-new-delivery', b'if(observe)o->child_event_pending=0;\n    return census(o,2,observe);',
        b'int observed=census(o,2,observe);if(observe)o->child_event_pending=0;return observed;', ['nested-census-delivery-control'])
    add('sample-old-combined-unit', b'return boundary==ENGINE_QUEUE_CENSUS_MEMBER||boundary==ENGINE_MEMBER_GS1;',
        b'(void)boundary;return 0;', ['sample-unit-composition-control'])
    add('batch-dropped-currentness-check', b'valid=private_guard_fresh(&private_guards.files[i]);',
        b'valid=i==0?private_fixed_check_end(clock_ns(CLOCK_THREAD_CPUTIME_ID)):private_guard_fresh(&private_guards.files[i]);', ['currentness-batch-control'])
    add('leader-refresh-use-cached', b'return o->members[1].terminal_observed_ns&&o->child_rollup_read_ns!=UINT64_MAX\n        &&o->child_rollup_read_ns>=o->members[1].terminal_observed_ns;',
        b'return o->members[0].phase==ENGINE_EXITED||(o->members[1].terminal_observed_ns&&o->child_rollup_read_ns!=UINT64_MAX\n        &&o->child_rollup_read_ns>=o->members[1].terminal_observed_ns);', ['leader-rollup-refresh-api', '3'])
    add('child-exit-ignore-note', b'if(event->fflags&NOTE_EXIT)', b'if(0&&(event->fflags&NOTE_EXIT))', ['child-exit-witness-api', '0'])
    add('child-exit-unbound-note', b'if(event->udata!=&o->event_watch[1]||!engine_child_watch_matches(o))',
        b'if(0&&(event->udata!=&o->event_watch[1]||!engine_child_watch_matches(o)))', ['child-exit-witness-api', '4'])
    add('verifier-pre-registration-run', b'!engine_events_register(o,0,raw.pid)', b'0', ['verifier-event-control', '0'])
    add('verifier-event-as-proof', b'o->leader_fork_events++;o->fork_census_pending=1;',
        b'o->leader_fork_events++;o->fork_census_pending=1;o->verified_descendants++;', ['verifier-event-api'])
    add('verifier-skip-terminal-drain', b'static int engine_events_service(struct engine_owner *o,uint64_t wait_ns) {',
        b'static int engine_events_service(struct engine_owner *o,uint64_t wait_ns) {int terminal=0;if(known_exited(o->ticket.pid,&terminal)&&terminal)return 1;', ['verifier-event-control', '0'])
    add('verifier-poll-only', b'if(event->fflags&NOTE_FORK)', b'if(0&&(event->fflags&NOTE_FORK))', ['verifier-event-control', '0'])
    add('verifier-ignore-registration', b'if(result!=0)return engine_failure(o,"coverage_unknown","process event registration failed");',
        b'if(0&&result!=0)return engine_failure(o,"coverage_unknown","process event registration failed");', ['verifier-event-control', '2'])
    add('verifier-early-queue-close', b'if(o->ticket.pid>0&&(!o->direct_reaped||!o->result->group_gone))return engine_failure(o,"settlement_unknown","process event queue retained until original ownreap and groupgone");',
        b'if(0&&o->ticket.pid>0&&(!o->direct_reaped||!o->result->group_gone))return engine_failure(o,"settlement_unknown","process event queue retained until original ownreap and groupgone");', ['verifier-event-api'])
    add('live-sample-closing-no-rescue', b'if(!control_observe(pid,&closing)) {\n        int read_error=errno;\n        if(engine_sample_terminal_after_error(owner,i,pid,read_error))return 1;',
        b'if(!control_observe(pid,&closing)) {\n        int read_error=errno;\n        if(0&&engine_sample_terminal_after_error(owner,i,pid,read_error))return 1;', ['live-sample-race-control', '6'])
    add('live-sample-errno-as-exit', b'if(engine_member_terminal(owner,i))return 1;', b'return 1;', ['live-sample-race-control', '5'])
    add('live-sample-no-recheck', b'if(read_error==ESRCH&&engine_sample_owned_member(owner,i,pid))',
        b'if(0&&read_error==ESRCH&&engine_sample_owned_member(owner,i,pid))', ['live-sample-race-control', '0'])
    add('live-sample-ignore-owned-row', b'if(read_error==ESRCH&&engine_sample_owned_member(owner,i,pid))',
        b'(void)pid;if(read_error==ESRCH)', ['live-sample-race-control', '2'])
    add('live-sample-any-error', b'if(read_error==ESRCH&&engine_sample_owned_member(owner,i,pid))',
        b'if(read_error!=0&&engine_sample_owned_member(owner,i,pid))', ['live-sample-race-control', '3'])
    add('process-create-old25', b'kind==NATIVE_PROCESS_CREATE?250000000:25000000',
        b'kind==NATIVE_PROCESS_CREATE?25000000:25000000', ['process-create-wall-control', '0'])
    add('process-create-ignore-cpu', b'return cpu<=2000000&&wall<=native_wall_limit(kind)&&inclusive<=3000000&&app<=1000000;',
        b'(void)cpu;return wall<=native_wall_limit(kind)&&inclusive<=3000000&&app<=1000000;', ['process-create-wall-control', '2'])
    add('argv-tree-literal-reject', b'if(strcmp(text,"^{tree}"))return 0;',
        b'if(1||strcmp(text,"^{tree}"))return 0;', ['argv-expansion-controls'])
    add('argv-unknown-brace-accepted', b'if(!value)return 0;', b'(void)0;', ['argv-expansion-controls'])
    add('helper-cpu-not-wall', b'uint64_t cost=engine_helper_cpu_cost(o->terminal_cpu_ns);',
        b'uint64_t cost=engine_interval(o->activation_ns,engine_now(o,CLOCK_MONOTONIC));if(cost<1000000)cost=1000000;', ['helper-cpu-debt-control'])
    add('total-helper-floor', b'static uint64_t engine_helper_cpu_cost(uint64_t cpu) {return cpu==UINT64_MAX?3000000:cpu<1000000?1000000:cpu;}',
        b'static uint64_t engine_helper_cpu_cost(uint64_t cpu) {return cpu==UINT64_MAX?3000000:cpu;}', ['total-boundary-api'])
    add('currentness-empty-floors', b'if(final&&leave_final_unpaid&&steps<63&&now>=begin&&now-begin<400000&&!private_open_last.native_failed)',
        b'if(final&&0&&leave_final_unpaid&&steps<63&&now>=begin&&now-begin<400000&&!private_open_last.native_failed)', ['terminal-deadline-controls'])
    add('terminal-descriptor-hash', b'&&private_same_file(&before,&after)&&!strcmp(hash,expected_hash);',
        b'&&private_same_file(&before,&after)&&(expected_hash!=NULL);', ['terminal-deadline-controls'])
    add('terminal-work-deadline', b'if(!private_cost.reporting_only)return 0;',
        b'if(!private_cost.reporting_only)return 1;', ['terminal-deadline-controls'])
    add('terminal-deadline-output', b'private_cost.reporting_only=1;',
        b'private_cost.reporting_only=0;', ['terminal-deadline-controls'])
    add('path-component-floor', b'if(++steps==64||now<begin||now-begin>=400000||final||private_open_last.native_failed)',
        b'if(++steps>=1||now<begin||now-begin>=400000||final||private_open_last.native_failed)', ['terminal-deadline-controls'])
    add('total-qos', b'||posix_spawnattr_set_qos_class_np(&p->attr,QOS_CLASS_BACKGROUND)!=0', b'||0', ['total-control', '0'])
    add('total-live-only', b'if(o->terminal_cpu_ns>ENGINE_COMMAND_CPU_NS||o->result->helper_job_cpu_ns>ENGINE_JOB_CPU_NS||(rss_known&&o->wait_rss>ENGINE_HELPER_RSS))',
        b'if(o->live_cpu_ns>ENGINE_COMMAND_CPU_NS||o->result->helper_job_cpu_ns>ENGINE_JOB_CPU_NS||(rss_known&&o->wait_rss>ENGINE_HELPER_RSS))', ['total-control', '1'])
    add('total-drop-rollup', b'o->terminal_cpu_ns=engine_wait4_cpu(&o->retained_wait);', b'o->terminal_cpu_ns=o->last_leader_self_ns;', ['total-control', '2'])
    add('total-no-live-kill', b'owner->live_over_cap=1;(void)engine_kill_group(owner);owner->live_kill_attempted=owner->group_kill_attempted;',
        b'owner->live_over_cap=1;owner->live_kill_attempted=0;', ['total-control', '3'])
    add('total-skip-helper-debt', b'static int engine_helper_payment(struct engine_owner *o) {',
        b'static int engine_helper_payment(struct engine_owner *o) { o->helper_cost_paid=1;return 1;', ['total-control', '4'])
    add('total-no-foreground-stop', b'int sent=kill(-o->ticket.pgid,SIGSTOP);', b'int sent=0;', ['total-control', '5'])
    add('total-false-eperm-settlement', b'if(!o->direct_reaped||!engine_original_group_gone(o))return 0;',
        b'if(o->group_kill_pending){o->result->group_gone=o->result->retired_coverage=1;return 1;}\n    if(!o->direct_reaped||!engine_original_group_gone(o))return 0;', ['total-control', '6'])
    add('total-stale-settlement-unknown', b'if(o->group_kill_pending) {o->group_kill_raced_exit=1;o->group_kill_pending=0;}',
        b'if(o->group_kill_pending) {o->group_kill_raced_exit=1;o->group_kill_pending=0;engine_failure(o,"settlement_unknown","mutant stale terminal unknown");}', ['total-control', '7'])
    add('total-terminal-rss', b'||(rss_known&&o->wait_rss>ENGINE_HELPER_RSS))', b'||0)', ['total-control', '8'])
    add('total-sample-gap', b'gap>ENGINE_SAMPLE_NS', b'0', ['total-boundary-api'])
    add('total-rest-slice', b'static uint64_t private_rest_slice_ns(uint64_t remaining) {return remaining>1000000?1000000:remaining;}', b'static uint64_t private_rest_slice_ns(uint64_t remaining) {return remaining;}', ['total-boundary-api'])
    add('total-ri-return-gap', b'int pass=engine_sample_gap(o,ended);if(!pass)o->ri_gap_failed_mask|=bit;return pass;', b'int pass=1;if(!pass)o->ri_gap_failed_mask|=bit;return pass;', ['total-retained-guard-api'])
    add('total-terminal-leader-skip-child', b'static int engine_final_kill_required(int original_terminal,int members_terminal) {return !(original_terminal&&members_terminal);}', b'static int engine_final_kill_required(int original_terminal,int members_terminal) {(void)members_terminal;return !original_terminal;}', ['total-retained-guard-api'])
    add('total-escaped-token-cleanup', b'static int engine_token_cleanup_required(int current,int birth_matches,int outside_group) {return current&&birth_matches&&outside_group;}', b'static int engine_token_cleanup_required(int current,int birth_matches,int outside_group) {(void)current;(void)birth_matches;(void)outside_group;return 0;}', ['total-retained-guard-api'])
    add('total-child-unproven', b'static int engine_child_cost_covered(const struct engine_owner *o) {return !o->child_cost_unproven&&(!o->child_created||o->child_rollup_known);}', b'static int engine_child_cost_covered(const struct engine_owner *o) {(void)o;return 1;}', ['total-retained-guard-api'])
    add('total-child-rollup-disappearance', b'return mask==31&&(user||system)&&user>=child_user&&system>=child_system;', b'(void)mask;(void)user;(void)system;(void)child_user;(void)child_system;return 1;', ['total-retained-guard-api'])
    add('total-native-false-empty', b'static int native_pid_list_empty(int count,int error) {return count==0&&error==0;}', b'static int native_pid_list_empty(int count,int error) {(void)error;return count==0;}', ['total-retained-guard-api'])
    pause_anchor=b'static int engine_pause_user_allowed(uint64_t before,uint64_t after,uint64_t resolution) {'
    pause_mutant=pause_anchor+b'(void)before;(void)after;(void)resolution;return 1;'
    add('total-pause-terminal-user', pause_anchor, pause_mutant, ['total-control', '9'])
    add('total-pause-live-user', pause_anchor, pause_mutant, ['total-control', '10'])
    add('total-unknown-rss', b'int pause_pass=engine_terminal_pause_user(o),rss_known=o->wait_rss!=UINT64_MAX;', b'int pause_pass=engine_terminal_pause_user(o),rss_known=1;', ['total-control', '11'])
    add('total-negative-rss-as-zero', b'static uint64_t engine_rss_value(long raw) {return raw<0?UINT64_MAX:(uint64_t)raw;}', b'static uint64_t engine_rss_value(long raw) {return raw<0?0:(uint64_t)raw;}', ['total-boundary-api'])
    add('skipped-engine', DISPATCH, b'(void)context;(void)result;return 0;', ['result-controls'])
    add('hardcoded-refusal', VALIDATED, VALIDATED+b'\n    private_fail("review_unbound","mutant fixed refusal after validation");goto refused;', ['flow-controls'])
    anchor=b'while(!engine_hold_observe(o))usleep(10000);'
    exit_trace=(b'fprintf(stderr,"private_hold_causal_exit_source_known controller=%d helper=%d quarantined=%u refusal_attempted=%u refusal_complete=%u own_wait4=%u group_gone=%u runtime_approved=false\\n",getpid(),o->ticket.pid,o->settlement_quarantined,o->refusal_attempted,o->refusal_complete,o->direct_reaped,o->result->group_gone);(void)fflush(stderr);_exit(81);')
    add('quarantine-exit', anchor, exit_trace+anchor, ['hold-lifetime-control'])
    add('quarantine-fd-release', anchor, b'if(o->input>=0){close(o->input);o->input=-1;}'+anchor, ['hold-lifetime-control'])
    add('hold-unpaid-observation', b'if(!private_pay_pending()||private_has_retained_cost()||private_cost.retained_overflow||private_cost.debt_paid!=private_cost.debt) {usleep(27000);return 0;}',
        b'(void)0;', ['hold-lifetime-control'])
    add('terminal-first-error-only', b'private_record_error(code,detail);', b'if(!private_failed)private_record_error(code,detail);', ['terminal-diagnostic-controls'])
    add('terminal-false-persistence', b'private_terminal.persist_ok=private_terminal.persist_open&&private_terminal.persist_write&&private_terminal.persist_fsync\n        &&private_terminal.persist_close&&private_terminal.persist_dir_fsync;',
        b'private_terminal.persist_ok=1;', ['terminal-diagnostic-controls'])
    add('terminal-skip-tail-cost', b'if(private_terminal.started&&!ok) {', b'if(0&&private_terminal.started&&!ok) {', ['terminal-diagnostic-controls'])
    start=source.index(b'    fprintf(stderr,"private_candidate_exit_tail ')
    end=source.index(b'    _Static_assert(11u*2304u',start)
    final=source.index(b'    char counts[512];',end)
    final_end=source.index(b'\n}\nstatic int private_early_refusal', final)
    early=source[:start]+source[end:final]+source[final:final_end]+source[start:end]+source[final_end:]
    result['terminal-before-tail']=(early,['terminal-diagnostic-controls'])
    add('engine-drop-inclusive-tail', b'else if(inclusive>actual) {', b'else if(0&&inclusive>actual) {', ['charge-diagnostic-control'])
    add('ordinary-before-debt', b'private_cost.last_unit=used;\n    if(used>private_cost.ordinary_max)',
        b'private_cost.last_unit=used;if(used>1000000)return private_fail("resource_limit","mutant early ordinary verdict before cost");\n    if(used>private_cost.ordinary_max)', ['diagnostic-setup-budget-control'])
    return result


def verifier_event_lifetime(stdout):
    raw=re.findall(rb'^private_verifier_child_lifetime_source_known child_pid=([0-9]+) own_WNOWAIT=1 ri_result=0 ri_errno=([0-9]+) start_abstime=([0-9]+) exit_abstime=([0-9]+) lifetime_ns=([0-9]+) timebase_numer=([0-9]+) timebase_denom=([0-9]+)\n',stdout,re.MULTILINE)
    axes=re.findall(rb'^private_verifier_event_lifetime_axis_source_known original_pid=([0-9]+) child_pid=([0-9]+) known=1 normal_leader_exit=1 poll_before_abstime=([0-9]+) child_start_abstime=([0-9]+) child_exit_abstime=([0-9]+) poll_after_abstime=([0-9]+) lifetime_ns=([0-9]+) strictly_between_polls=1 actual_own_wait4=1 group_gone=1 runtime_approved=false\n',stdout,re.MULTILINE)
    if len(raw)!=1 or len(axes)!=1:
        raise ValueError('actual kernel fast-child lifetime and original settled-owner interval receipts missing or duplicated')
    child,error,start,exit_at,lifetime,numer,denom=map(int,raw[0])
    original,axis_child,before,axis_start,axis_exit,after,axis_lifetime=map(int,axes[0])
    if (not original or not child or child==original or child!=axis_child or start!=axis_start or exit_at!=axis_exit
            or not before<start<exit_at<after or not numer or not denom
            or lifetime!=axis_lifetime or lifetime!=((exit_at-start)*numer+denom-1)//denom
            or not 0<lifetime):
        raise ValueError('actual kernel child lifetime is unknown or not strictly between actual census polls')
    return {'original_pid':original,'child_pid':child,'ri_errno_on_success':error,'raw_start_abstime':start,
            'raw_exit_abstime':exit_at,'timebase_numer':numer,'timebase_denom':denom,
            'rounded_up_lifetime_ns':lifetime,'poll_before_abstime':before,'poll_after_abstime':after,
            'actual_child_WNOWAIT_before_own_parent_reap':True,'actual_original_wait4_and_groupgone':True,
            'scope':'source-known actual fork lifetime; not installed Git/GPG qualification'}


def child_exit_witness_receipt(stdout):
    rows=re.findall(rb'^private_child_exit_witness_process_source_known original_pid=([0-9]+) child_pid=([0-9]+) RI_result=(-?[0-9]+) RI_errno=([0-9]+) child_exit_witness=kqueue_note_exit child_exit_abstime=0 terminal_observed_ns=([0-9]+) cached_leader_RI_ns=([0-9]+) first_leader_terminal_ns=([0-9]+) later_leader_RI_ns=([0-9]+) rollup_mask=31 child_cost_unproven=0 actual_leader_own_wait4=1 original_group_gone=1 controller_reaps_child=false leader_child_wait4_CPU_ns=([0-9]+) authoritative_subtree_CPU_ns=([0-9]+) helper_debt_delta_ns=([0-9]+) helper_debt_before_ns=([0-9]+) helper_debt_target_ns=([0-9]+) global_paid_ns=([0-9]+) GS1_fixture_projection=true installed_tool_qualification=false runtime_approved=false\n',stdout,re.MULTILINE)
    if len(rows)!=1:raise ValueError('actual leader-reaped child knote witness receipt missing or duplicated')
    leader,child,result,error,terminal,cached,first,read,child_CPU,CPU,delta,before,target,paid=map(int,rows[0])
    if (not leader or not child or leader==child or result==0 or error!=errno.ESRCH or not 0<first<=cached<terminal<=read
            or CPU<child_CPU or delta!=9*max(1000000,CPU) or target!=before+delta or paid<target):
        raise ValueError('child knote witness lacks RI_ESRCH, post-terminal rollup, single subtree bill or full9x payment')
    return {'original_pid':leader,'child_pid':child,'RI_error':error,'child_exit_witness':'kqueue_note_exit',
            'exit_abstime':0,'terminal_observed_ns':terminal,'cached_leader_RI_ns':cached,'first_leader_terminal_ns':first,'later_leader_RI_ns':read,'rollup_mask':31,
            'child_wait4_done_by_original_leader':True,'controller_child_wait4':False,
            'raw_child_CPU_ns':child_CPU,'authoritative_subtree_CPU_ns':CPU,'helper_debt_delta_ns':delta,
            'helper_global_debt_before_ns':before,'helper_global_target_ns':target,'global_paid_ns':paid,
            'scope':'actual source-known process/knote/rollup/accounting; GS1 admission supplied, no installed tool qualification'}


def retain_current(stdout, report):
    found=re.findall(rb'private_engine_controls=pass cases=101 total_budget_controls=8 synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=(/private/tmp/dream-private-engine-controls\.[A-Za-z0-9]+)\n',stdout)
    if len(found)!=1:
        raise ValueError('current fixture lacks exact101 remaining controls and eight total-budget witnesses')
    if os.fsdecode(found[0]) != report['commands'][-1]['fixture_capture']['source_dir']:
        raise ValueError('completed fixture root differs from original command binding')
    modes=re.findall(rb'private_total_budget_control_source_known mode=([0-9]+) original_pid=([0-9]+) actual_own_wait4=1 group_gone=1 terminal_cpu_ns=([0-9]+)',stdout)
    if sorted(int(x[0]) for x in modes)!=list(range(12)):
        raise ValueError('twelve original ownwait4/groupgone witnesses missing or duplicated')
    report['total_budget_owned_process_witnesses']=[{'mode':int(m),'original_pid':int(pid),'authoritative_rounded_wait4_ns':int(cpu),
        'scope':'mandated_decision66280' if int(m)<8 else 'additional_terminal_RSS_or_paused_user_boundary'} for m,pid,cpu in modes]
    debt = re.findall(rb'private_helper_cpu_debt_source_known original_pid=([0-9]+) actual_own_wait4=1 group_gone=1 wall_ns=([0-9]+) CPU_ns=([0-9]+) helper_cost_ns=([0-9]+) helper_debt_before_ns=([0-9]+) helper_debt_delta_ns=([0-9]+) helper_debt_target_ns=([0-9]+) global_paid_after_helper_ns=([0-9]+)',stdout)
    if len(debt)!=1:
        raise ValueError('actual original long-wall helper CPU debt witness missing or duplicated')
    pid,wall,cpu,cost,before,delta,target,paid=map(int,debt[0])
    if wall<100000000 or wall<=2*cpu or cost!=max(1000000,cpu) or delta!=9*cost or target!=before+delta or not before or paid<target:
        raise ValueError('helper CPU delta is conflated with elapsed wall or the cumulative controller baseline')
    report['helper_cpu_debt_owned_process_witness']={'pid':pid,'wall_ns':wall,'authoritative_CPU_ns':cpu,'helper_delta_ns':delta,'global_debt_before_ns':before,'global_target_ns':target,'global_paid_after_ns':paid,'scope':'actual owned fixture, not installed tool qualification'}
    argv_marker=b'private_argv_expansion_source_known actual_registry_form=10 prepared_valid=true prepared_corrupt=true selector_literal=true unknown_braces_refused=true vector_scope=production_prepare_and_expand native_process=false runtime_approved=false\n'
    if stdout.count(argv_marker)!=1:
        raise ValueError('production registry/preparation argv tree-selector control missing or duplicated')
    flows=re.findall(rb'private_flow_cost_source_known variant=([0-3]) actual_argv_prepares=15 prepared_forms=32767 entry_ns=([0-9]+) end_ns=([0-9]+) elapsed_ns=([0-9]+) original_deadline_ns=([0-9]+)',stdout)
    if sorted(int(row[0]) for row in flows)!=list(range(4)):
        raise ValueError('all four source-known full15 actual argv engine flow costs required')
    for variant,start,end,elapsed,deadline in flows:
        start,end,elapsed,deadline=map(int,(start,end,elapsed,deadline))
        if end<start or end-start!=elapsed or end>=deadline or deadline-start!=30000000000:
            raise ValueError('source-known engine flow violated original30s timing or raw elapsed identity')
    report['source_known_argv_and_engine_scope']={'actual_registry_forms':list(range(15)),'actual_prepares_per_variant':15,'variants':4,'timing_raw_scope':'command_engine_only; real controller/config/reference/argv/semantic work, native/wait4 scripted; outer-inner authentication/platform validation not exercised','actual_helper_births':0,'whole_private_candidate_fit_proven':False}
    create_rows=re.findall(rb'private_process_create_completed_source_known mode=([0-2]) actual_own_wait4=1 group_gone=1 policy_inputs_projected=true expected_exit=([0-9]+)',stdout)
    if sorted((int(m),int(code)) for m,code in create_rows)!=[(0,0),(1,81),(2,81)]:
        raise ValueError('three actual owned-fixture creation policy projections lack completed original settlement')
    report['process_create_policy_controls']={'supplied_policy_inputs':True,'actual_OS_costs_separately_retained':True,'actual_owned_effects_and_settlement':True,'OS_34ms_or_251ms_timing_claimed':False,'positive_wall_ns':34000000,'positive_excess_ns':9000000,'refused_wall_ns':251000000,'refused_excess_ns':226000000,'cpu_above_2ms_refused':True,'other_native_wall_ns':25000000,'inclusive_ns':3000000}
    race_rows=re.findall(rb'private_live_sample_race_completed_source_known mode=([0-6]) actual_own_wait4=1 group_gone=1 expected_exit=([0-9]+) terminal_fields_changed=false',stdout)
    if sorted((int(m),int(code)) for m,code in race_rows)!=[(0,0),(1,81),(2,81),(3,81),(4,81),(5,81),(6,0)]:
        raise ValueError('first-read ESRCH race cases lack complete original ownwait4 and group settlement')
    report['live_read_race_controls']={'modes':{'0':'actual gated original exits after first precheck; supplied readESRCH; qualified terminal recheck','1':'supplied foreign birth still coverage_unknown','2':'supplied mismatched full audit token still coverage_unknown','3':'nonESRCH EIO never retries despite actual terminal proof','4':'successful TASKALLINFO then injected clock failure sanitizes staleESRCH','5':'ESRCH on still-live exact original requires actual terminal proof and refuses','6':'actual exit injected at closing TASKALLINFO uses same rescue'},'actual_original_wait4_and_groupgone':True,'actual_OS_read_result_and_applied_fault_separate':True,'rescue_sites':['first TASKALLINFO','closing TASKALLINFO'],'successful_closing_lineage_and_QoS_checks':'unchanged','terminal_fields_changed':False,'real_candidate_created':0}
    event_rows=re.findall(rb'private_verifier_event_completed_source_known mode=([0-2]) actual_own_wait4=1 group_gone=1 expected_exit=81',stdout)
    if sorted(map(int,event_rows))!=[0,1,2]:
        raise ValueError('three actual original event-discovery refusals lack complete settlement')
    if stdout.count(b'private_verifier_event_api_source_known scripted_child_lifetime_ns=100000 notifications_coalesce=true native_process=false actual_queue_lifetime=true runtime_approved=false\n')!=1:
        raise ValueError('scripted submillisecond event/census API or actual queue-lifetime witness missing')
    report['verifier_event_actual_kernel_lifetime']=verifier_event_lifetime(stdout)
    report['verifier_event_controls']={'scripted_between_poll_child_lifetime_ns':100000,'scripted_times_are_OS_claims':False,
        'owned_process_modes':{'0':'actual kernel RI start/exit measured lifetime strictly between actual polls; child reaped by its own parent; actual queued event yields honest missed-verifier refusal',
                               '1':'actual never-fork leader; distinct created-no-verifier refusal',
                               '2':'injected registration EIO before CONT; actual original effect settled'},
        'notifications_are_birth_counts':False,'GS1_or_whole_lifetime_proof_from_events':False,
        'queue':'actual CLOEXEC fd retained until original ownwait4/groupgone and charged close',
        'terminal_fields_changed':False,'ledger_fields':{'leader_fork_events':'u64','leader_exec_events':'u64','child_discovered_via':'none|poll|fork_event','fork_seen_child_missed':'bool'}}
    child_api=re.findall(rb'private_child_exit_witness_API_completed_source_known mode=([0-8]) expected_exit=([0-9]+) native_process=false',stdout)
    if sorted((int(m),int(code)) for m,code in child_api)!=[(i,81 if i else 0) for i in range(9)]:
        raise ValueError('child exit witness production API positive and guard negatives incomplete')
    report['child_exit_witness_actual_process']=child_exit_witness_receipt(stdout)
    report['child_exit_witness_API']={'scope':'supplied process/watch/errno/counter state; no child birth or physical child reap claim',
        'cases':{'0':'process-bound watch spans approved exec; first receipt stable; NOTE_EXIT plus ESRCH qualifies',
                 '1':'NOTE_EXEC alone refuses','2':'failed registration/no registered child watch refuses',
                 '3':'foreign watch birth refuses','4':'forged userdata refuses','5':'nonESRCH RI refuses',
                 '6':'rollup less than sampled child CPU refuses','7':'prior sticky child-cost-unproven remains refused',
                 '8':'token not gone refuses'},'exit_abstime_not_fabricated':True,'new_terminal_fields':{},
        'new_ledger_field':{'child_exit_witness':'ri|kqueue_note_exit|none'}}
    refresh_api=re.findall(rb'private_leader_rollup_refresh_API_completed_source_known mode=([0-4]) expected_exit=([0-9]+) native_process=false',stdout)
    if sorted((int(m),int(code)) for m,code in refresh_api)!=[(i,81 if i else 0) for i in range(5)]:
        raise ValueError('fresh original leader RI validity/cache guard API cases incomplete')
    report['leader_rollup_refresh_API']={'scope':'supplied cached terminal owner, current-token identity, RI status/payload and timestamps; no native process',
        'cases':{'0':'genuine fresh post-child terminal observation passes strict raw guards','1':'missing RI refuses',
                 '2':'foreign RI start refuses','3':'cached terminal and RI read before child receipt are stale',
                 '4':'reaped original owner refuses'},'cached_success_not_new_observation':True}
    composition=re.findall(rb'^private_sample_unit_composition_source_known supplied_costs=true native_process=false retained_total_ns=1020958 composed_total_ns=1020958 phase_units=3 phase_max_ns=([0-9]+) nominal_target_ns=500000 hard_unit_ns=1000000 projected_debt_ns=27000000 OS_WCET_claimed=false runtime_approved=false\n',stdout,re.MULTILINE)
    if len(composition)!=1 or int(composition[0])>500000:
        raise ValueError('shared source unit-boundary composition control missing or above nominal bound')
    batches=re.findall(rb'^private_currentness_batch_source_known guards=2 expected_fixed_checks=8 completed_checks=8 units_closed=([0-9]+) old_min_units=9 max_reserved_steps=([0-9]+) unit_cpu_max_ns=([0-9]+) active_after=0 debt_ns=([0-9]+) paid_ns=([0-9]+) actual_file_checks=true platform_checks=true real_candidate_created=0 runtime_approved=false\n',stdout,re.MULTILINE)
    if len(batches)!=1:raise ValueError('actual bounded currentness batching witness missing or duplicated')
    units,steps,cpu,debt,paid=map(int,batches[0])
    if not 0<units<9 or not 0<steps<=64 or cpu>1000000 or debt!=paid:
        raise ValueError('currentness batch omitted fixed checks, exceeded step/cost bounds or retained unpaid work')
    if b'private_currentness_batch_change_source_known actual_in_place_change=true refused=1 original_descriptors_closed=1' not in stdout:
        raise ValueError('actual retained-file mutation refusal and paid descriptor cleanup missing')
    report['unit_composition_control']={'supplied_stress_total_ns':1020958,'retained_row_phase_attribution_proven':False,
        'groups':3,'nominal_max_ns':int(composition[0]),'hard_ordinary_ns':1000000,'OS_WCET_claimed':False}
    report['currentness_batch_control']={'actual_roots':3,'actual_retained_files':2,'actual_platform_checks':3,'fixed_checks':8,
        'closed_units':units,'old_min_units':9,'max_reserved_steps':steps,'actual_max_unit_CPU_ns':cpu,
        'all_actual_debt_paid':True,'actual_in_place_change_refused':True,'real_candidate_created':0,
        'whole_positive_or_corrupt_30s_margin_proven':False}
    nested=b'private_nested_census_delivery_source_known cases=3 old_coalesced_consumed=true nested_EXEC_EXIT_retained=true native_process=false runtime_approved=false\n'
    if stdout.count(nested)!=1:
        raise ValueError('old coalesced/new nested census delivery production-span guard missing')
    report['nested_census_delivery_control']={'cases':['new NOTE_EXEC','new NOTE_EXIT','no new delivery'],
        'old_pending_consumed_before_observing_census':True,'new_nested_delivery_retained_after_census':True,
        'source_scope':'shared observing-census span +production event-note identity guard; supplied identities/notes, no native process',
        'new_exit_note_not_own_wait4_or_GS1':True}
    report['baseline_cases']=101
    marker=b'private_total_retained_guard_source_known api=true supplied_clock_and_identity_values=true native_process=false runtime_approved=false\n'
    if stdout.count(marker)!=1:
        raise ValueError('current retained-clock/child production API witness missing or duplicated')
    report['retained_guard_API']='supplied scalar clock/identity preconditions; no physical orphan or child wait4 claim'
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    if b'private_quarantine_hold_source_known ' not in stdout:
        raise ValueError('current same-controller prompt/refusal/FD/owner-loss witness missing')


def verify(directory, inputs, report, total_only=False):
    source=inputs[str(SOURCE)]
    current=directory/'current';current.mkdir(mode=0o700)
    for name in (SOURCE,CONTROLS):
        report['snapshots'].append(retain(current/name.name,inputs[str(name)]))
    program=compile_source(current,CONTROLS.name,report)
    if total_only:
        for mode in range(12):
            code,stdout,stderr=run([str(program),'total-control',str(mode)],current,'total-'+str(mode),report,fixture=True)
            expect(code,0,'current total-budget mode'+str(mode))
            if (f'private_total_budget_control_source_known mode={mode} ').encode() not in stdout:
                raise ValueError('missing actual total-budget witness'+str(mode))
        code,stdout,_=run([str(program),'total-retained-guard-api'],current,'retained-guard-api',report,fixture=True)
        expect(code,0,'current retained-clock/child production API')
    else:
        code,stdout,stderr=run([str(program)],current,'controls',report,fixture=True)
        expect(code,0,'current remaining source-known controls')
        retain_current(stdout,report)
        code,_,_=run([str(program),'diagnostic-setup-budget-control'],current,'own-thread-negative',report,fixture=True)
        expect(code,0,'actual over-budget own-thread cost/debt negative')
        for mode in (1,2):
            code,stdout,stderr=run([str(program),'process-create-wall-control',str(mode)],current,'process-create-refusal-'+str(mode),report,fixture=True)
            expect(code,81,'owned process-create policy refusal '+str(mode))
            marker=(f'private_process_create_completed_source_known mode={mode} actual_own_wait4=1 group_gone=1 policy_inputs_projected=true expected_exit=81').encode()
            if marker not in stdout or b'probe refused:' in stderr:
                raise ValueError('native policy refusal did not finish the intended causal assertion and actual owner cleanup')
    if not total_only:
        for mode in (1,2,3,4,5):
            code,stdout,stderr=run([str(program),'live-sample-race-control',str(mode)],current,'live-sample-refusal-'+str(mode),report,fixture=True)
            expect(code,81,'owned first-read live-sample refusal '+str(mode))
            marker=(f'private_live_sample_race_completed_source_known mode={mode} actual_own_wait4=1 group_gone=1 expected_exit=81 terminal_fields_changed=false').encode()
            if marker not in stdout or b'probe refused:' in stderr:
                raise ValueError('live-sample refusal lacks its intended assertion and original owner cleanup')
    if not total_only:
        code,stdout,stderr=run([str(program),'live-sample-race-control','6'],current,'live-sample-closing-positive',report,fixture=True)
        expect(code,0,'owned closing TASKALLINFO terminal rescue')
        if b'private_live_sample_race_completed_source_known mode=6 actual_own_wait4=1 group_gone=1 expected_exit=0' not in stdout:
            raise ValueError('closing sample rescue lacks its actual original owner completion')
    if not total_only:
        for mode in range(3):
            code,stdout,stderr=run([str(program),'verifier-event-control',str(mode)],current,'verifier-event-refusal-'+str(mode),report,fixture=True)
            expect(code,81,'owned event-discovery refusal '+str(mode))
            marker=(f'private_verifier_event_completed_source_known mode={mode} actual_own_wait4=1 group_gone=1 expected_exit=81').encode()
            if marker not in stdout or b'probe refused:' in stderr:
                raise ValueError('event refusal lacks its intended diagnostic and original owner cleanup')
            if mode==0:report['commands'][-1]['actual_kernel_child_lifetime']=verifier_event_lifetime(stdout)
    if not total_only:
        for mode in range(1,9):
            code,stdout,stderr=run([str(program),'child-exit-witness-api',str(mode)],current,'child-exit-witness-refusal-'+str(mode),report,fixture=True)
            expect(code,81,'child exit witness API refusal '+str(mode))
            marker=(f'private_child_exit_witness_API_completed_source_known mode={mode} expected_exit=81 native_process=false').encode()
            if marker not in stdout or b'probe refused:' in stderr:
                raise ValueError('child exit witness API negative did not complete exact intended refusal/latch assertions')
    if not total_only:
        for mode in range(1,5):
            code,stdout,stderr=run([str(program),'leader-rollup-refresh-api',str(mode)],current,'leader-rollup-refresh-refusal-'+str(mode),report,fixture=True)
            expect(code,81,'fresh leader RI API refusal '+str(mode))
            marker=(f'private_leader_rollup_refresh_API_completed_source_known mode={mode} expected_exit=81 native_process=false').encode()
            if marker not in stdout or b'probe refused:' in stderr:
                raise ValueError('fresh leader RI API negative lacks exact intended cache/identity/clock refusal')
    all_variants=variants(source)
    selected={k:v for k,v in all_variants.items() if not total_only or k.startswith('total-')}
    for name,(changed_source,args) in selected.items():
        own=directory/name;own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own/SOURCE.name,changed_source))
        report['snapshots'].append(retain(own/CONTROLS.name,inputs[str(CONTROLS)]))
        mutant=compile_source(own,CONTROLS.name,report)
        code,stdout,stderr=run([str(mutant),*args],own,'controls',report,fixture=True)
        expect(code,81,'causal negative '+name)
        if name.startswith('total-') and args[0]=='total-control' and name != 'total-qos':
            if b'private_total_property_axis_source_known ' not in stdout:
                raise ValueError('mutant did not reach independent native/unit/physical/causal prerequisites: '+name)
        if ASSERTIONS[name] not in stderr:
            raise ValueError('mutant missed its required causal assertion: '+name)
        if name in ('verifier-poll-only','verifier-skip-terminal-drain'):
            report['commands'][-1]['actual_kernel_child_lifetime']=verifier_event_lifetime(stdout)
        if name=='total-qos' and (b'detail=live helper priority differs from observed background band4' not in stderr
                or b'actual_reaped=1 group_gone=1' not in stderr):
            raise ValueError('missing-QoS mutation refused for another reason or lacks own cleanup')
        if name in ('quarantine-exit','quarantine-fd-release','hold-unpaid-observation'):
            verify_lifetime_mutant_cleanup(stderr,report['commands'][-1])
        if name=='quarantine-exit':
            verify_controller_liveness_mutant(stderr,report['commands'][-1])
        report.setdefault('causal_mutants',[]).append({'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),'scope':'source-known API/owned fixture, never installed Git/GPG qualification'})
    if not total_only:
        production=directory/'production';production.mkdir(mode=0o700)
        report['snapshots'].append(retain(production/SOURCE.name,source))
        probe=compile_source(production,SOURCE.name,report)
        code,_,_=run([str(probe),'git',*(['unsupported']*8)],production,'normal-git-unproven',report)
        expect(code,81,'normal unsupported pair remains refused')
        code,_,_=run([str(probe),'private-candidate','/nonexistent-q2-source-known-input','0'*64],production,'malformed-prebirth',report)
        expect(code,81,'malformed private input refuses before birth')
    report['source_mutants']=len(selected)
    report['retired_external_budget_machinery']={'decision':66280,'per_grant_windows_adaptive_T_N':'removed, not passed',
        'historical_C18_inputs':'target/dream-read-q2-restart-01M6RZZXSHZT3KHBK1QC7K3XB3/retained-C18-before-66280/preservation.json',
        'all_historical_failed_runs':'remain failed', 'legacy_late_stop_case':'strict codec retained; explicit review_unbound before birth, no new runtime witness or invented meaning'}


def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir',required=True)
    parser.add_argument('--total-budget-only',action='store_true',help='supplemental eight66280 plus four RSS/paused-user API/owned-process controls; not complete current C receipt')
    args=parser.parse_args();target=Path(args.target_dir)
    if not target.is_absolute():parser.error('--target-dir must be absolute')
    target.mkdir(parents=True,exist_ok=True)
    directory=Path(tempfile.mkdtemp(prefix='private-engine-total-budget-' if args.total_budget_only else 'private-engine-controls-',dir=target)).resolve()
    report={'format':'bboxy.dream.private-engine-source-controls/1','decision':66280,'scope':'supplemental_total_budget' if args.total_budget_only else 'canonical_remaining_C_controls','synthetic_source_known':True,'real_candidate_created':0,'runtime_approved':False,'whole_command_lifetime_proof':False,'live_kernel_custody_after_probe_exit_proved':False,'recovery_handoff_proved':False,'same_controller_quarantine_hold_witness_required':True,'commands':[],'inputs':{},'snapshots':[],'executables':[],'process_ledgers':[]}
    status=1
    try:
        inputs={str(p):read_stable(ROOT/p,1024*1024) for p in (SOURCE,CONTROLS,DRIVER)}
        archive=directory/'inputs';archive.mkdir(mode=0o700)
        for name,data in inputs.items():report['inputs'][name]=retain(archive/Path(name).name,data)
        verify(directory,inputs,report,args.total_budget_only)
        if any(read_stable(ROOT/name,1024*1024)!=data for name,data in inputs.items()):raise ValueError('canonical C3 changed during verification')
        report['current_sources_unchanged']=True;report['status']='passed';status=0
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        report['status']='failed';report['error']=str(error)
    retain(directory/'run.json',canonical(report))
    print(json.dumps({'status':report['status'],'evidence':str(directory/'run.json'),'real_candidate_created':0,'runtime_approved':False},sort_keys=True))
    return status


if __name__=='__main__':
    sys.exit(main())
