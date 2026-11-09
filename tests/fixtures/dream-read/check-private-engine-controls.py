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
# Task66445 fixed production observation boundaries. A per-site control must
# reach that boundary's forced raw receipt; no direct shared-helper success is
# accepted as caller-wiring evidence. UNIX and Mach result domains stay distinct.
OBSERVATION_SITES = (
    'live_taskall_open',
    'audit_taskall_before',
    'audit_task_name',
    'audit_task_name_null',
    'audit_task_info',
    'audit_taskall_after',
    'live_image_path',
    'live_rusage',
    'live_taskall_close',
    'gs1_image_path',
    'gs1_procargs',
    'child_stop_signal',
    'child_continue_signal',
    'initial_continue_signal',
    'foreground_stop_signal',
    'foreground_continue_signal',
    'prekill_bsd',
    'final_group_kill',
    'children_census',
    'group_census',
    'event_register',
    'event_poll',
    'event_delivery',
    'handoff_rusage',
    'escaped_cleanup_bsd',
    'escaped_cleanup_signal',
    'owned_terminal_waitid',
)
OBSERVATION_ASSERTION = b'selected production observation boundary retains qualified terminal-only disposition'

VALIDATED = (b'if(!engine_owner_priority()||!engine_pretest_current(context)'
             b'||!engine_signature_boundary_available(context))goto refused;')

# Eight decision66280 controls plus independent preserved own-thread, semantic,
# persistence and original-controller custody controls. No retired graph/control
# is re-labelled as a current pass.
LEDGERS = ('final-conjunction-controls.ndjson', 'member-peak-census/total-6.ndjson',
           *(f'flow-{i}/ledger.ndjson' for i in range(4)),
           'hold-ledger.ndjson', 'hold-control-success', 'claim.json', 'charge-diagnostic.ndjson',
           'same-wake/total-27.ndjson', 'same-wake/admission-child', 'same-wake/slow-exec-delay', 'same-wake/execute-fifo.json', 'same-wake/exit',
           'admitted-descriptor', 'retained-descriptor', 'currentness-a', 'currentness-b', 'deadline-result/stdout', 'deadline-result/result.json',
           *(f'total-{i}.ndjson' for i in range(27)), 'knote-rollup-26', *(f'live-race-{i}' for i in range(16,23)), *(f'event-release-{i}' for i in range(23,26)), 'rollup-2', 'rollup-2-terminal', 'rollup-2-release', 'release-6',
           *(f'pause-{i}-{phase}' for i in (9,10) for phase in ('start','done','release')),
           *('diagnostic-' + str(i) + '/' + name for i in range(4) for name in ('stderr', 'stdout', 'result.json')))
ASSERTIONS = {
 'pause-stop-no-reap-category': b'pending STOP without ownwait4 records coverage_unknown and preserves the first independent error and custody',
 'pause-stop-exit-ignore-path': b'ESRCH terminal-before-stop API settles only after original ownwait4 groupgone and full known paid bill',
 'total-stopped-old-raw-gap': b'stable ACKed stop over50ms retains raw gap and admits both bounded running sides',
 'total-stopped-no-ACK': b'unACKed STOP earns no cadence exclusion even with supplied stable counters',
 'total-stopped-ignore-SYSTEM': b'SYSTEM beyond50us invalidates the immutable ACK counter seal',
 'total-stopped-per-member-only': b'aggregate SYSTEM30000 plus20001ns refuses despite each member below50us',
 'total-stopped-return-gap': b'post-resume RI query-return overshoot refuses and preserves every raw endpoint',
 'total-stopped-no-fresh-end': b'resume without a fresh stopped RI endpoint cannot supply stopped credit',
 'member-peak-drop-admission': b'supplied current member admissions reach exact live peak and final conjunction',
 'final-conjunction-drop-term': b'final conjunction ledger contains every boolean term and first failing name',
 'custody-stop-ignore-current-RI': b'fresh current RI and sticky admission state gate the one verifier custody STOP',
 'same-wake-nonblocking-only': b'bounded event wait admits slow-exec projection without exhausting the unchanged CPU guard',
 'same-wake-post-stop-nonblocking-only': b'bounded post-STOP wait admits delayed ACK projection before the unchanged CPU guard',
 'same-wake-insert-pre-stop-rest': b'same-wake admission API keeps pre-STOP window open until positive custody ACK',
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
 'total-pause-terminal-user': b'USER-only terminal production API refuses excess with unchanged SYSTEM',
 'total-pause-live-user': b'USER-only live production API refuses growth with unchanged SYSTEM',
 'total-unknown-rss': b'unknown terminal RSS stays unavailable while full known wait4 CPU debt is paid',
 'total-negative-rss-as-zero': b'negative terminal RSS API remains unknown rather than zero or a finite breach',
 'total-terminal-rss': b'actual terminal wait4 RSS above16MiB refuses after ownreap and full CPU payment',
 'total-sample-gap': b'known running and unACKed paused RI gaps over50ms refuse without resetting prior evidence',
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

FINAL_TERMS = ('no_prior_failure','engine_entered','semantic_forms_complete','commands_complete','attempts_complete',
    'created_complete','reaped_complete','members_max_complete','group_gone','retired_coverage','no_live_owner',
    'terminal_CPU_checked_complete','terminal_CPU_passed_complete','proof_matches_case','ordinary_within_1ms',
    'controller_within_1ms','cleanup_within_1ms','native_CPU_within_2ms','native_wall_within_process_create_limit',
    'inclusive_within_3ms','helper_job_within_3s','A_within_3s','C_known_finite','helper_RSS_within_16MiB',
    'probe_RSS_within_96MiB','no_retained_cost','debt_fully_paid','ordinary_raw_within_1ms')


def final_conjunction_records(path):
    records=[]
    for line in read_stable(path, 2*1024*1024).splitlines():
        pairs=json.loads(line,object_pairs_hook=list)
        if not isinstance(pairs,list) or len({k for k,v in pairs})!=len(pairs):
            raise ValueError('final conjunction ledger duplicate key or non-object record')
        row=dict(pairs)
        if row.get('kind')!='final_case_conjunction':continue
        expected=set(FINAL_TERMS)|{'kind','capture_cutoff','coverage_passed','resources_passed','first_failing_term','runtime_approved'}
        if set(row)!=expected or any(type(row[k]) is not bool for k in FINAL_TERMS):
            raise ValueError('final conjunction ledger must contain all28 exact independent bool terms')
        first=next((k for k in FINAL_TERMS if not row[k]),'none')
        if (row['capture_cutoff']!='before_this_diagnostic_format_write_fsync_close'
                or type(row['coverage_passed']) is not bool or type(row['resources_passed']) is not bool
                or row['coverage_passed']!=all(row[k] for k in FINAL_TERMS[:14])
                or row['resources_passed']!=all(row[k] for k in FINAL_TERMS[14:])
                or row['first_failing_term']!=first or row['runtime_approved'] is not False):
            raise ValueError('final conjunction first failure/group results/cutoff differ from independent terms')
        records.append(row)
    return records


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



def observation_production_bindings(source):
    # This is a source-bound caller map, not a second C interpreter. Actual
    # matrix commands enter the same raw wrappers and prove selected receipts;
    # these checks also reject an omitted/mis-tagged enclosing production call.
    mask = re.sub(rb"""/\*.*?\*/|//[^\n]*|"(?:\\.|[^"\\])*"|'(?:\\.|[^'\\])*'""",
                  lambda m: bytes(10 if b == 10 else 32 for b in m.group()), source, flags=re.DOTALL)
    def body(name):
        pattern = rb'(?m)^static\b[^;{}]*\b' + name.encode() + rb'\([^;{}]*\)\s*\{'
        matches = list(re.finditer(pattern, mask))
        if len(matches) != 1:
            raise ValueError('one production function body required: ' + name)
        start = matches[0].start(); opening = matches[0].end() - 1; depth = 1; end = opening + 1
        while end < len(mask) and depth:
            depth += (mask[end] == ord('{')) - (mask[end] == ord('}'));end += 1
        if depth:
            raise ValueError('incomplete production function body: ' + name)
        return source[start:end], source[:start].count(b'\n') + 1
    maps = [
        ('live_taskall_open', 'engine_sample_member', ('ENGINE_OBS_LIVE_TASKALL_OPEN',)),
        *[(name, 'known_control_token_observed', ('ENGINE_OBS_' + name.upper(),)) for name in OBSERVATION_SITES[1:6]],
        ('live_image_path', 'engine_sample_member', ('engine_observe_path', 'ENGINE_OBS_LIVE_IMAGE_PATH')),
        ('live_rusage', 'engine_sample_member', ('engine_observe_rusage', 'ENGINE_OBS_LIVE_RUSAGE')),
        ('live_taskall_close', 'engine_sample_member', ('ENGINE_OBS_LIVE_TASKALL_CLOSE',)),
        ('gs1_image_path', 'private_git_sig_gs1_owned', ('engine_observe_path', 'ENGINE_OBS_GS1_IMAGE_PATH')),
        ('gs1_procargs', 'private_git_sig_gs1_owned', ('engine_observe_procargs',)),
        ('child_stop_signal', 'engine_child_send_stop', ('engine_observe_token_signal', 'ENGINE_OBS_CHILD_STOP_SIGNAL')),
        ('child_continue_signal', 'engine_sample_member', ('engine_observe_token_signal', 'ENGINE_OBS_CHILD_CONTINUE_SIGNAL')),
        ('initial_continue_signal', 'engine_birth', ('engine_observe_token_signal', 'ENGINE_OBS_INITIAL_CONTINUE_SIGNAL')),
        ('foreground_stop_signal', 'engine_pause', ('engine_observe_group_signal', 'ENGINE_OBS_FOREGROUND_STOP_SIGNAL')),
        ('foreground_continue_signal', 'engine_resume', ('engine_observe_group_signal', 'ENGINE_OBS_FOREGROUND_CONTINUE_SIGNAL')),
        ('prekill_bsd', 'engine_kill_group', ('engine_observe_bsd', 'ENGINE_OBS_PREKILL_BSD')),
        ('final_group_kill', 'engine_kill_group', ('engine_observe_group_signal', 'ENGINE_OBS_FINAL_GROUP_KILL')),
        ('children_census', 'engine_list_children', ('engine_observation_fault', 'ENGINE_OBS_CHILDREN_CENSUS', 'native_child_count')),
        ('group_census', 'engine_list_group', ('engine_observation_fault', 'ENGINE_OBS_GROUP_CENSUS', 'native_group_count')),
        ('event_register', 'engine_events_register', ('engine_observation_fault', 'ENGINE_OBS_EVENT_REGISTER', 'kevent')),
        ('event_poll', 'engine_events_poll', ('engine_observation_fault', 'ENGINE_OBS_EVENT_POLL', 'kevent')),
        ('event_delivery', 'engine_event_note', ('engine_observation_fault', 'ENGINE_OBS_EVENT_DELIVERY')),
        ('handoff_rusage', 'engine_child_handoff_wait', ('engine_observe_rusage', 'ENGINE_OBS_HANDOFF_RUSAGE')),
        ('escaped_cleanup_bsd', 'engine_cleanup_escaped', ('engine_observe_bsd', 'ENGINE_OBS_ESCAPED_CLEANUP_BSD')),
        ('escaped_cleanup_signal', 'engine_cleanup_escaped', ('engine_observe_token_signal', 'ENGINE_OBS_ESCAPED_CLEANUP_SIGNAL')),
        ('owned_terminal_waitid', 'engine_observe_exited', ('engine_observation_fault', 'ENGINE_OBS_OWNED_TERMINAL_WAITID', 'known_exited')),
    ]
    records = []
    if [row[0] for row in maps] != list(OBSERVATION_SITES):
        raise ValueError('production binding map differs from fixed observation inventory')
    for site, name, required in maps:
        data, line = body(name)
        if any(token.encode() not in data for token in required):
            raise ValueError('production enclosing caller missing fixed raw boundary: ' + site)
        records.append({'site': site, 'production_function': name, 'line': line, 'function_sha256': sha(data),
                        'required_source_tokens': list(required),
                        'scope': 'source binding; runtime matrix separately exercises actual wrapper plus supplied phase where labelled'})
    # TERMINAL2/PENDING3 must not be promoted through a generic truthy enum.
    if re.search(rb'if\s*\(\s*!?\s*(?:engine_observation_route|observe_or_terminal)\s*\(', mask):
        raise ValueError('terminal outcome used as a native-data boolean')
    if re.search(rb'if\s*\(\s*!?\s*[A-Za-z_][A-Za-z_0-9]*\.outcome\s*\)', mask):
        raise ValueError('terminal enum2 must be distinguished from live native-data enum1')
    return records


def observation_variants(source):
    result = {}
    for number, site in enumerate(OBSERVATION_SITES, 1):
        pattern = rb'ENGINE_OBSERVATION_ROUTE\(' + site.encode() + rb'\s*,\s*ENGINE_OBS_' + site.upper().encode() + rb'\)'
        matches = list(re.finditer(pattern, source))
        if len(matches) != 1:
            raise ValueError('one production observation route required: ' + site)
        anchor = matches[0].group()
        name = 'observation-bypass-' + site
        ASSERTIONS[name] = OBSERVATION_ASSERTION
        result[name] = (changed(source, anchor, anchor.replace(b'ENGINE_OBSERVATION_ROUTE', b'ENGINE_OBSERVATION_BYPASS', 1)),
                        ['observation-race-control', str(number), '0'])
    name = 'observation-advisory-wnowait-gate'
    ASSERTIONS[name] = b'advisory own-WNOWAIT not-ready does not veto independent original terminal RI proof'
    result[name] = (changed(source, b'(void)waitable;return qualified;', b'(void)waitable;return qualified&&waitable;'),
                    ['observation-race-control', '1', '7'])
    extra = [
        ('observation-group-census-fatal',
         b'n=engine_list_group(owner,group,sizeof(group));\n    if(owner->obs_terminal_pending)return 0;',
         b'n=engine_list_group(owner,group,sizeof(group));\n    if(0&&owner->obs_terminal_pending)return 0;',
         ['observation-race-control','20','0'], OBSERVATION_ASSERTION),
        ('observation-resume-first-fatal',
         b'int sampled=engine_sample_member(o,i,o->members[i].state.pid);\n        if(o->obs_terminal_pending)return 0;\n        if(!sampled||o->members[i].state.status!=SSTOP',
         b'int sampled=engine_sample_member(o,i,o->members[i].state.pid);\n        if(!sampled||o->obs_terminal_pending||o->members[i].state.status!=SSTOP',
         ['observation-resume-control','1'], b'resume sample terminal takeover stops before live predicates without false coverage failure'),
        ('observation-resume-fresh-fatal',
         b'int sampled=(o->pause_counter_known_mask&(1u<<i))?engine_sample_member(o,i,o->members[i].state.pid):0;\n        if(o->obs_terminal_pending)return 0;\n        if(!(o->pause_counter_known_mask&(1u<<i))||!sampled',
         b'int sampled=(o->pause_counter_known_mask&(1u<<i))?engine_sample_member(o,i,o->members[i].state.pid):0;\n        if(!(o->pause_counter_known_mask&(1u<<i))||!sampled||o->obs_terminal_pending',
         ['observation-resume-control','2'], b'resume sample terminal takeover stops before live predicates without false coverage failure'),
        ('observation-event-eintr-success',
         b'if(n<0) {\n        if(error==EINTR&&engine_cancel_signal)',
         b'if(n<0) {if(error==EINTR)return 1;\n        if(error==EINTR&&engine_cancel_signal)',
         ['observation-event-interrupt-control','0'], b'event EINTR refuses through exact site coverage while preserving an already observed cancellation first'),
    ]
    extra.append(('observation-normal-terminal-kill-skip',
                  b'if(owner->obs_terminal_pending&&terminal&&engine_group_observed_terminal(owner,1))',
                  b'if(terminal&&engine_group_observed_terminal(owner,1))',
                  ['total-control','7'],
                  b'natural-exit pending final kill resolves errno after ownreap groupgone without stale unknown'))
    extra.append(('observation-terminal-user-sys-early',
                  b'static int engine_terminal_pause_user(struct engine_owner *o) {',
                  b'static int engine_terminal_pause_user(struct engine_owner *o) {if(o->pause_system_violation)return 0;',
                  ['terminal-both-policy-api'],
                  b'terminal USER diagnosis survives simultaneous strict SYSTEM refusal and preserves earlier FIRST error'))
    for name, anchor, replacement, args, assertion in extra:
        ASSERTIONS[name] = assertion
        result[name] = (changed(source,anchor,replacement),args)
    return result


def observation_race_receipt(stdout, number, case=0, *, mutant=False, fatal_guard=False):
    prefix = b'private_observation_race_source_known '
    rows = [dict(part.decode('ascii').split('=', 1) for part in line[len(prefix):].split())
            for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows) != 1:
        raise ValueError('one selected production observation receipt required')
    row = rows[0]
    if (row.get('enclosing_phase_supplied') != ('true' if number > 9 and number != 20 else 'false')
            or row.get('actual_GS1') != 'false' or row.get('actual_escaped_child') != 'false'):
        raise ValueError('wrapper projection must not claim whole GS1 or escaped-child execution')
    if (int(row['site_id']) != number or row['site'] != OBSERVATION_SITES[number - 1]
            or int(row['case']) != case or row['boundary_hits'] != '1' or int(row['seen_site']) != number
            or row['raw_error_supplied'] != 'true' or row['target_syscall_invoked'] != 'false'
            or row['syscall_outcome_supplied'] != 'true' or row['runtime_approved'] != 'false'):
        raise ValueError('selected raw production boundary/site provenance differs')
    if (row['advisory_OS_window_claimed'] != 'false' or row['consuming_WNOWAIT_fault'] != 'false'
            or (case == 7 and row['advisory_waitable_supplied'] != '0')):
        raise ValueError('advisory classification must not fake consuming WNOWAIT or claim a native publication window')
    for field in ('ACK_unchanged', 'CONT_unchanged', 'membership_unchanged', 'census_unknown_preserved',
                  'watch_not_fabricated', 'GS1_not_fabricated'):
        if row[field] != 'true':
            raise ValueError('observation recovery invented live success: ' + field)
    accepted = case in (0, 7) and not mutant
    qualified = accepted or fatal_guard
    before, after = int(row['selected_takeovers_before']), int(row['selected_takeovers_after'])
    later = int(row['later_takeovers_before_retire'])
    if (row['pipeline_snapshot'] != 'before_retirement' or row['selected_callbacks'] != '1'
            or int(row['selected_site']) != number or row['selected_domain'] != row['raw_domain']
            or row['selected_code'] != row['raw_code'] or after < before
            or int(row['selected_takeover_delta']) != after - before
            or later < 0 or int(row['takeovers']) != after + later):
        raise ValueError('selected route-return snapshot or later cumulative provenance differs')
    if (int(row['selected_outcome']) != (2 if qualified else 0)
            or int(row['selected_pending']) != int(qualified)
            or after - before != int(qualified)
            or bool(int(row['selected_owner_failed'])) == qualified
            or bool(int(row['selected_coverage'])) == qualified
            or (not qualified and row['selected_first_site_coverage'] != '1')):
        raise ValueError('selected production route outcome does not match terminal proof/premises')
    if accepted:
        # Local success never replaces the original whole-caller/cost verdict.
        if (row['terminal_pending'] != '1' or row['takeovers'] != '1'
                or row['coverage_unknown'] != '0' or row['pipeline_owner_failed'] != '0' or later):
            raise ValueError('nominal observation later violated its whole-caller success guards')
    elif fatal_guard:
        if row['terminal_pending'] != '1' or not int(row['coverage_unknown']) or row['pipeline_owner_failed'] != '1':
            raise ValueError('restored outer fatal guard did not refuse its selected terminal takeover')
    elif mutant:
        # A later independent strict-cadence cleanup may legitimately take over
        # another site. It cannot stand in for this bypassed selected route.
        if not int(row['coverage_unknown']) or row['pipeline_owner_failed'] != '1':
            raise ValueError('selected bypass lacks retained whole-caller coverage refusal')
    elif row['terminal_pending'] != '0' or row['takeovers'] != '0' or not int(row['coverage_unknown']):
        raise ValueError('negative observation premises did not preserve whole-caller refusal')
    if row['native_process'] == 'true':
        if (row['terminal_proof_supplied'] != 'false' or row['actual_own_wait4'] != '1'
                or row['group_gone'] != '1' or row['retired_coverage'] != '1'
                or row['helper_cost_retained'] != '1' or row['helper_cost_paid'] != '1'
                or int(row['original_pid']) <= 0 or int(row['CPU_ns']) >= 2**64 - 1
                or int(row['helper_debt_ns']) != 9 * max(1000000, int(row['CPU_ns']))
                or row['debt_ns'] != row['paid_ns']):
            raise ValueError('observation process lacks original ownwait4/groupgone/full known CPU debt')
        if case != 4 and (int(row['terminal_start']) <= 0 or int(row['terminal_exit']) <= 0):
            raise ValueError('observation process lacks independent original terminal RI')
    elif row['native_process'] != 'false' or row['terminal_proof_supplied'] != 'true' or row['actual_own_wait4'] != 'false':
        raise ValueError('supplied observation API overclaims native process or ownwait4')
    return row


def changed(source, anchor, replacement):
    if source.count(anchor) != 1:
        raise ValueError('causal mutation anchor differs: ' + repr(anchor))
    return source.replace(anchor, replacement, 1)


def variants(source):
    result = {}
    def add(name, anchor, replacement, args):
        result[name] = (changed(source, anchor, replacement), args)
    add('pause-stop-no-reap-category', b'return engine_failure(o,"coverage_unknown","foreground STOP lacks original ownwait4 terminal settlement");',
        b'return 1;', ['pause-stop-exit-api'])
    add('pause-stop-exit-ignore-path', b'static int engine_pause_stop_error_eligible(int error) {return error==ESRCH||error==EPERM;}',
        b'static int engine_pause_stop_error_eligible(int error) {(void)error;return 0;}', ['pause-stop-exit-api'])
    add('total-stopped-old-raw-gap', b'if(running_gap==UINT64_MAX||running_gap>ENGINE_SAMPLE_NS)',
        b'if(gap==UINT64_MAX||gap>ENGINE_SAMPLE_NS)', ['stopped-gap-api'])
    add('total-stopped-no-ACK', b'!(o->pause_ack_mask&bit)', b'0', ['stopped-gap-api'])
    add('total-stopped-ignore-SYSTEM', b'return ns!=UINT64_MAX&&ns<=50000;',
        b'(void)ns;return 1;', ['stopped-gap-api'])
    add('total-stopped-per-member-only', b'if(!engine_pause_system_ns_allowed(o->stopped_system_ns)) {',
        b'if(!engine_pause_system_ns_allowed(ns)) {', ['stopped-gap-api'])
    add('total-stopped-return-gap', b'int pass=engine_sample_gap(o,ended);if(!pass)o->ri_gap_failed_mask|=bit;return pass;',
        b'int pass=1;if(!pass)o->ri_gap_failed_mask|=bit;return pass;', ['stopped-gap-api'])
    add('total-stopped-no-fresh-end', b'||resume_end<resume_begin', b'||0', ['stopped-gap-api'])
    add('member-peak-drop-admission', b'o->members[i].live_admitted=1;',
        b'if(i)return;o->members[i].live_admitted=1;', ['member-peak-admission-api'])
    add('final-conjunction-drop-term', b'n=snprintf(bytes+used,capacity-used,"\\\"%s\\\":%s,",engine_final_term_names[i],c->term[i]?"true":"false");',
        b'if(i==7)continue;n=snprintf(bytes+used,capacity-used,"\\\"%s\\\":%s,",engine_final_term_names[i],c->term[i]?"true":"false");', ['final-conjunction-controls'])
    add('custody-stop-ignore-current-RI', b'return !o->failed&&u&&u->ri_proc_start_abstime&&!u->ri_proc_exit_abstime\n        &&u->ri_proc_start_abstime==o->members[1].state.start_absolute\n        &&u->ri_proc_start_abstime==o->event_watch[1].start&&engine_child_watch_matches(o);',
        b'(void)o;(void)u;return 1;', ['custody-stop-identity-api'])
    add('same-wake-nonblocking-only', b'static uint64_t engine_child_handoff_event_wait(unsigned pending,unsigned stop_sent) {(void)stop_sent;return pending?0:1000000;}',
        b'static uint64_t engine_child_handoff_event_wait(unsigned pending,unsigned stop_sent) {(void)pending;(void)stop_sent;return 0;}', ['same-wake-event-wait-api'])
    add('same-wake-post-stop-nonblocking-only', b'static uint64_t engine_child_handoff_event_wait(unsigned pending,unsigned stop_sent) {(void)stop_sent;return pending?0:1000000;}',
        b'static uint64_t engine_child_handoff_event_wait(unsigned pending,unsigned stop_sent) {return pending||stop_sent?0:1000000;}', ['same-wake-stop-ack-api'])
    add('same-wake-insert-pre-stop-rest', b'return sample(o,1,pid);',
        b'int observed=sample(o,1,pid);return observed&&engine_checkpoint(o);', ['same-wake-admission-api'])
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
    add('verifier-ignore-registration', b'o->event_register_errno[i]=error;\n    if(result!=0){(void)engine_observation_route(o,i,&failure);return 0;}',
        b'o->event_register_errno[i]=error;\n    if(0&&result!=0){(void)engine_observation_route(o,i,&failure);return 0;}', ['verifier-event-control', '2'])
    add('verifier-early-queue-close', b'if(o->ticket.pid>0&&(!o->direct_reaped||!o->result->group_gone))return engine_failure(o,"settlement_unknown","process event queue retained until original ownreap and groupgone");',
        b'if(0&&o->ticket.pid>0&&(!o->direct_reaped||!o->result->group_gone))return engine_failure(o,"settlement_unknown","process event queue retained until original ownreap and groupgone");', ['verifier-event-api'])
    # Preserve the original first/closing/foreign/clock witnesses through the
    # new shared routes; only the old guard's implementation anchor changed.
    add('live-sample-closing-no-rescue', b'ENGINE_OBSERVATION_ROUTE(live_taskall_close, ENGINE_OBS_LIVE_TASKALL_CLOSE)',
        b'ENGINE_OBSERVATION_BYPASS(live_taskall_close, ENGINE_OBS_LIVE_TASKALL_CLOSE)', ['live-sample-race-control', '6'])
    add('live-sample-errno-as-exit', b'int terminal=engine_observation_terminal_classified(engine_member_terminal(o,i),0);',
        b'int terminal=engine_observation_terminal_classified(1,0);', ['live-sample-race-control', '5'])
    add('live-sample-no-recheck', b'ENGINE_OBSERVATION_ROUTE(live_taskall_open, ENGINE_OBS_LIVE_TASKALL_OPEN)',
        b'ENGINE_OBSERVATION_BYPASS(live_taskall_open, ENGINE_OBS_LIVE_TASKALL_OPEN)', ['live-sample-race-control', '0'])
    add('live-sample-ignore-owned-row', b'!engine_observation_identity_complete(o,i)',
        b'(site==ENGINE_OBS_LIVE_TASKALL_OPEN?0:!engine_observation_identity_complete(o,i))', ['live-sample-race-control', '2'])
    add('live-sample-any-error', b'(!pending&&!engine_observation_gone(site,domain,code))',
        b'(!pending&&!(site==ENGINE_OBS_LIVE_TASKALL_OPEN&&domain==ENGINE_OBSERVATION_UNIX&&code==EIO)&&!engine_observation_gone(site,domain,code))', ['live-sample-race-control', '3'])
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
    add('total-no-foreground-stop', b'struct engine_observed_signal stopped=engine_observe_group_signal(o,ENGINE_OBS_FOREGROUND_STOP_SIGNAL,SIGSTOP);',
        b'struct engine_observed_signal stopped={.result=0,.error=0,.returned_ns=engine_now(o,CLOCK_MONOTONIC),.outcome=ENGINE_OBSERVATION_LIVE};', ['total-control', '5'])
    add('total-false-eperm-settlement', b'if(!o->direct_reaped||!engine_original_group_gone(o))return 0;',
        b'if(o->group_kill_pending){o->result->group_gone=o->result->retired_coverage=1;return 1;}\n    if(!o->direct_reaped||!engine_original_group_gone(o))return 0;', ['total-control', '6'])
    add('total-stale-settlement-unknown', b'if(o->group_kill_pending) {o->group_kill_raced_exit=1;o->group_kill_pending=0;}',
        b'if(o->group_kill_pending) {o->group_kill_raced_exit=1;o->group_kill_pending=0;engine_failure(o,"settlement_unknown","mutant stale terminal unknown");}', ['total-control', '7'])
    add('total-terminal-rss', b'||(rss_known&&o->wait_rss>ENGINE_HELPER_RSS))', b'||0)', ['total-control', '8'])
    add('total-sample-gap', b'running_gap>ENGINE_SAMPLE_NS', b'0', ['total-boundary-api'])
    add('total-rest-slice', b'static uint64_t private_rest_slice_ns(uint64_t remaining) {return remaining>1000000?1000000:remaining;}', b'static uint64_t private_rest_slice_ns(uint64_t remaining) {return remaining;}', ['total-boundary-api'])
    add('total-ri-return-gap', b'int pass=engine_sample_gap(o,ended);if(!pass)o->ri_gap_failed_mask|=bit;return pass;', b'int pass=1;if(!pass)o->ri_gap_failed_mask|=bit;return pass;', ['total-retained-guard-api'])
    add('total-terminal-leader-skip-child', b'static int engine_final_kill_required(int original_terminal,int members_terminal) {return !(original_terminal&&members_terminal);}', b'static int engine_final_kill_required(int original_terminal,int members_terminal) {(void)members_terminal;return !original_terminal;}', ['total-retained-guard-api'])
    add('total-escaped-token-cleanup', b'static int engine_token_cleanup_required(int current,int birth_matches,int outside_group) {return current&&birth_matches&&outside_group;}', b'static int engine_token_cleanup_required(int current,int birth_matches,int outside_group) {(void)current;(void)birth_matches;(void)outside_group;return 0;}', ['total-retained-guard-api'])
    add('total-child-unproven', b'static int engine_child_cost_covered(const struct engine_owner *o) {return !o->child_cost_unproven&&(!o->child_created||o->child_rollup_known);}', b'static int engine_child_cost_covered(const struct engine_owner *o) {(void)o;return 1;}', ['total-retained-guard-api'])
    add('total-child-rollup-disappearance', b'return mask==31&&(user||system)&&user>=child_user&&system>=child_system;', b'(void)mask;(void)user;(void)system;(void)child_user;(void)child_system;return 1;', ['total-retained-guard-api'])
    add('total-native-false-empty', b'static int native_pid_list_empty(int count,int error) {return count==0&&error==0;}', b'static int native_pid_list_empty(int count,int error) {(void)error;return count==0;}', ['total-retained-guard-api'])
    pause_anchor=b'static int engine_pause_user_allowed(uint64_t before,uint64_t after,uint64_t resolution) {'
    pause_mutant=pause_anchor+b'(void)before;(void)after;(void)resolution;return 1;'
    add('total-pause-terminal-user', pause_anchor, pause_mutant, ['pause-user-api', '1'])
    add('total-pause-live-user', pause_anchor, pause_mutant, ['pause-user-api', '0'])
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


def stopped_cadence_receipt(stdout):
    rows=re.findall(rb'^private_stopped_gap_process_source_known ([^\n]+)\n',stdout,re.MULTILINE)
    if len(rows)!=1:raise ValueError('actual positively stopped cadence process receipt missing or duplicated')
    pairs=[part.split(b'=',1) for part in rows[0].split()]
    if any(len(p)!=2 for p in pairs) or len({p[0] for p in pairs})!=len(pairs):
        raise ValueError('stopped cadence receipt has malformed or duplicate fields')
    row=dict(pairs)
    names=('original_pid','ack_ns','ack_RI_begin_ns','ack_RI_return_ns','resume_RI_begin_ns','resume_RI_return_ns',
        'resume_signal_ns','ACK_USER_raw','end_USER_raw','ACK_SYSTEM_raw','end_SYSTEM_raw','stopped_system_ns',
        'system_timebase_numer','system_timebase_denom','stopped_interval_ns',
        'raw_gap_max_ns','running_gap_max_ns','helper_CPU_ns','helper_debt_delta_ns','debt_ns','paid_ns')
    if any(not re.fullmatch(rb'[0-9]+',row.get(k.encode(),b'')) for k in names):
        raise ValueError('stopped cadence numeric endpoints/counters unavailable')
    n={k:int(row[k.encode()]) for k in names}
    required={b'actual_own_wait4':b'1',b'group_gone':b'1',b'original_FD_closed':b'true',
        b'native_process':b'true',b'installed_tool_qualification':b'false',b'runtime_approved':b'false'}
    if (any(row.get(k)!=v for k,v in required.items()) or not n['original_pid']
            or not 0<n['ack_ns']<=n['ack_RI_begin_ns']<=n['ack_RI_return_ns']<=n['resume_RI_begin_ns']<=n['resume_RI_return_ns']<=n['resume_signal_ns']
            or n['ACK_USER_raw']!=n['end_USER_raw'] or n['end_SYSTEM_raw']<n['ACK_SYSTEM_raw']
            or not n['system_timebase_numer'] or not n['system_timebase_denom']
            or n['stopped_system_ns']!=((n['end_SYSTEM_raw']-n['ACK_SYSTEM_raw'])*n['system_timebase_numer']+n['system_timebase_denom']-1)//n['system_timebase_denom']
            or n['stopped_system_ns']>50000 or n['helper_CPU_ns']<n['stopped_system_ns']
            or n['stopped_interval_ns']!=n['resume_signal_ns']-n['ack_ns'] or n['stopped_interval_ns']<=50000000
            or n['raw_gap_max_ns']<=50000000 or n['running_gap_max_ns']>50000000
            or n['helper_debt_delta_ns']!=9*max(1000000,n['helper_CPU_ns']) or n['debt_ns']!=n['paid_ns']):
        raise ValueError('actual stopped cadence lacks exact USER/bounded aggregate SYSTEM, exact ACK/resume cut, bounded running sides or full paid settlement')
    return dict(n,scope='actual owned STOP/RI/resume/ownwait4/groupgone; no installed tool qualification',
        ledger_fields={'stopped_interval_ns':'u64','running_gap_max_ns':'u64','stopped_system_ns':'u64'},
        nonzero_stopped_SYSTEM_observed=n['stopped_system_ns']>0,SYSTEM_inside_full_wait4_CPU_not_subtracted=True,runtime_approved=False)


def retain_current(stdout, report):
    found=re.findall(rb'private_engine_controls=pass cases=114 total_budget_controls=8 synthetic_source_known=true real_candidate_created=0 runtime_approved=false retained=(/private/tmp/dream-private-engine-controls\.[A-Za-z0-9]+)\n',stdout)
    if len(found)!=1:
        raise ValueError('current fixture lacks exact114 remaining controls and eight total-budget witnesses')
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
    admission=re.findall(rb'^private_same_wake_admission_completed_source_known original_pid=([0-9]+) child_pid=([0-9]+) child_image_admitted=1 GS1_bound=1 GS2_settled=1 rollup_mask=31 actual_own_wait4=1 group_gone=1 helper_CPU_ns=([0-9]+) helper_debt_delta_ns=([0-9]+) debt_ns=([0-9]+) paid_ns=([0-9]+) native_process=true installed_tool_qualification=false runtime_approved=false\n',stdout,re.MULTILINE)
    stopped=re.findall(rb'^private_same_wake_STOP_axis_source_known positive_ACK_before_first_rest=1 actual_full_unit_CPU_ns=([0-9]+) hard_unit_ns=1000000 nominal_target_ns=500000 no_OS_WCET_claim=true runtime_approved=false\n',stdout,re.MULTILINE)
    api=b'private_same_wake_admission_API_source_known captured=1 closed_paid_unit_before_ACK=0 supplied_stopped_ACK=1 supplied_GS1=1 supplied_rollup_mask=31 sticky_child_cost_unproven=0 native_process=false values_supplied=true actual_GS1=false actual_own_wait4=false runtime_approved=false\n'
    if len(admission)!=1 or len(stopped)!=1 or stdout.count(api)!=1:
        raise ValueError('same-wake actual GS1 and separate API ordering witnesses missing or duplicated')
    leader,child,cpu,delta,debt,paid=map(int,admission[0]);unit=int(stopped[0])
    if not leader or not child or leader==child or delta!=9*max(1000000,cpu) or debt!=paid or not 0<unit<=1000000:
        raise ValueError('same-wake actual owner full cost/debt/STOP unit prerequisites failed')
    wakeup=re.findall(rb'^private_same_wake_execute_wakeup_source_known prepared_before_birth=true original_FIFO_dev=([0-9]+) original_FIFO_ino=([0-9]+) write_result=1 write_errno=0 write_begin_CPU_ns=([0-9]+) write_end_CPU_ns=([0-9]+) child_uses_blocking_read=true child_execute_timer_poll=false controller_write_nonblocking=true runtime_approved=false\n',stdout,re.MULTILINE)
    if len(wakeup)!=1:
        raise ValueError('actual prepared FIFO execute wakeup receipt missing or duplicated')
    fifo_dev,fifo_ino,write_begin,write_end=map(int,wakeup[0])
    if not fifo_dev or not fifo_ino or not 0<write_begin<=write_end:
        raise ValueError('actual prepared FIFO identity or nonblocking write clocks unknown')
    slow=re.findall(rb'^private_same_wake_slow_exec_source_known child_pid=([0-9]+) clone_begin_ns=([0-9]+) clone_end_ns=([0-9]+) clone_delay_ns=([0-9]+) minimum_delay_ns=750000 actual_clock_reads=true actual_GS1=true native_process=true installed_tool_qualification=false runtime_approved=false\n',stdout,re.MULTILINE)
    if len(slow)!=1:
        raise ValueError('actual slow-CLONE measured delay receipt missing or duplicated')
    slow_pid,clone_begin,clone_end,clone_delay=map(int,slow[0])
    if slow_pid!=child or not 0<clone_begin<=clone_end or clone_delay!=clone_end-clone_begin or clone_delay<750000:
        raise ValueError('actual original slow-CLONE did not remain before EXEC for measured750us')
    projected=b'private_same_wake_event_wait_API_source_known admitted=1 supplied_CPU_ns=220000 supplied_elapsed_ns=750000 supplied_steps=1 supplied_rollup_mask=31 sticky_child_cost_unproven=0 native_process=false values_supplied=true OS_CPU_claimed=false runtime_approved=false\n'
    terminal=b'private_same_wake_exit_API_source_known EXEC_only_not_terminal=1 retained_EXIT_received=1 image_admitted=0 custody_STOP=0 sticky_child_cost_unproven=1 distinct_detail="verifier exited before admission" native_process=false values_supplied=true actual_GS1=false actual_own_wait4=false runtime_approved=false\n'
    if stdout.count(projected)!=1 or stdout.count(terminal)!=1:
        raise ValueError('event-wait and exact pre-admission EXIT API projections missing or duplicated')
    report['same_wake_admission_control']={'original_pid':leader,'child_pid':child,'actual_GS1_and_GS2':True,
        'actual_clone_delay_ns':clone_delay,'clone_delay_clock_begin_ns':clone_begin,'clone_delay_clock_end_ns':clone_end,
        'prepared_execute_FIFO':{'device':fifo_dev,'inode':fifo_ino,'actual_nonblocking_write_CPU_ns':write_end-write_begin,'child_execute_timer_poll':False},
        'post_ACK_full_unit_CPU_ns':unit,'hard_unit_ns':1000000,'nominal_target_ns':500000,'OS_WCET_claimed':False,
        'authoritative_subtree_CPU_ns':cpu,'helper_debt_delta_ns':delta,'actual_own_wait4_and_groupgone':True,
        'API_negative_scope':'supplied-state correlated gap at shared production capture span; no physical missing-GS1 quarantine exit',
        'installed_tool_qualification':False}
    identity=b'private_custody_STOP_identity_API_source_known cases=7 accepted_mask=1 expected_mask=1 foreign_RI_zero_RI_terminal_RI_failed_admission_noncurrent_row_wrong_watch_refused=true native_process=false values_supplied=true actual_SIGSTOP=false runtime_approved=false\n'
    if stdout.count(identity)!=1:
        raise ValueError('fresh-RI/failed-admission custody-STOP guard API witness missing or duplicated')
    report['custody_STOP_identity_API']={'cases':7,'accepted_mask':1,'expected_mask':1,
        'source_scope':'shared production pre-SIGSTOP guard; supplied current/foreign/zero/terminal RI and retained identity rows, no actual signal',
        'old_refusal_passive_settlement_observations':'preserved; newly failed precheck returns before new dispatch'}
    members=b'private_same_wake_members_conjunction_source_known actual_members_max=2 census_peak_path_bypassed=true actual_GS1_GS2_settled=true actual_own_wait4_groupgone=true actual_full_bill_paid=true final_conjunction_passed=true first_failing_term="none" final_error_code=none other_final_terms_supplied=true full15_physical_run_claimed=false runtime_approved=false\n'
    census=b'private_members_peak_census_source_known actual_group_count=1 members_max=1 actual_own_wait4=1 group_gone=1 all_cost_paid=true native_process=true runtime_approved=false\n'
    diagnostic=b'private_final_conjunction_API_source_known rows=3 terms=28 actual_owned_FD=true no_path_reopen=true all_cost_paid=true cutoff=before_this_diagnostic_format_write_fsync_close native_process=false runtime_approved=false\n'
    if stdout.count(members)!=1 or stdout.count(census)!=1 or stdout.count(diagnostic)!=1:
        raise ValueError('real same-wake/census peaks and owned-FD conjunction serialization witnesses missing')
    evidence=report['commands'][-1]['fixture_capture']['files']
    final_path=next((Path(f['path']) for f in evidence if f['path'].endswith('/final-conjunction-controls.ndjson')),None)
    if final_path is None:raise ValueError('actual owned final-conjunction API ledger not retained')
    term_rows=final_conjunction_records(final_path)
    expected_false=[set(),{'members_max_complete','group_gone'},
        {'inclusive_within_3ms','helper_job_within_3s','helper_RSS_within_16MiB'}]
    if len(term_rows)!=3 or any({k for k in FINAL_TERMS if not row[k]}!=bad for row,bad in zip(term_rows,expected_false)):
        raise ValueError('final conjunction API lost later independent failures or changed bool semantics')
    for variant in (0,3):
        path=next((Path(f['path']) for f in evidence if f['path'].endswith(f'/flow-{variant}/ledger.ndjson')),None)
        rows=final_conjunction_records(path) if path else []
        if len(rows)!=1 or rows[0]['first_failing_term']!='none':
            raise ValueError('actual functional source-known full15 flow lost original-FD final diagnostics')
    report['member_peak_and_final_conjunction']={'actual_same_wake_peak':2,'actual_census_only_peak':1,
        'actual_GS1_GS2_ownwait4_groupgone_full_bill':True,'other_full15_terms_in_single_helper_projection':'supplied, not physical',
        'ledger_bool_terms':list(FINAL_TERMS),'first_failure_field':'first_failing_term:string',
        'snapshot_cutoff':'before_this_diagnostic_format_write_fsync_close','writer_tail_all_costs_paid_before_completion':True,
        'API_rows':3,'source_known_full15_integration_variants':[0,3],'runtime_approved':False}
    member_api=b'private_members_peak_admission_API_source_known cases=1 retained_live_count=2 members_max=2 exact_current_rows_supplied=2 final_conjunction_passed=true first_failing_term="none" final_error_code=none other_final_terms_supplied=true helper_births=0 native_process=false values_supplied=true actual_GS1=false actual_GS2=false actual_own_wait4=false actual_group_gone=false actual_helper_bill=false OS_timing_claimed=false full15_physical_run_claimed=false runtime_approved=false\n'
    if stdout.count(member_api)!=1:raise ValueError('supplied production member admission/count/final-classifier API positive missing or duplicated')
    report['member_peak_admission_API']={'cases':1,'retained_live_count':2,'members_max':2,'native_process':False,'values_supplied':True,'actual_own_wait4':False,'other_final_terms':'supplied; no full15 physical claim'}
    report['baseline_cases']=114
    report['pause_stop_exit_API']=pause_stop_exit_api_receipt(stdout)
    stop_ack=b'private_stop_ACK_API_source_known case=delayed_ACK admitted=1 supplied_CPU_ns=260000 supplied_STOP_to_ACK_ns=3000100 supplied_steps=3 sticky_child_cost_unproven=0 native_process=false values_supplied=true actual_STOP=false actual_native_wait=false actual_own_wait4=false OS_latency_claimed=false runtime_approved=false\n'
    never_ack=b'private_stop_ACK_API_source_known case=never_ACK at_bound=1 past_bound=0 supplied_STOP_elapsed_ns=25000001 remaining_request_ns=500 distinct_detail="stop ACK not observed within bound" native_process=false values_supplied=true actual_STOP=false actual_native_wait=false actual_own_wait4=false OS_latency_claimed=false runtime_approved=false\n'
    if stdout.count(stop_ack)!=1 or stdout.count(never_ack)!=1:raise ValueError('supplied post-STOP delayed/never ACK causal APIs missing or misclassified')
    report['post_STOP_ACK_API']={'grouped_cases':3,'native_process':False,'values_supplied':True,'actual_STOP':False,'actual_native_wait':False,'OS_latency_claimed':False,'bound_ns':25000000,'exact_bound_positive':True,'past_bound_refusal_detail':'stop ACK not observed within bound'}
    report['stopped_cadence_actual_process']=stopped_cadence_receipt(stdout)
    USER_rows=re.findall(rb'private_pause_USER_only_API_source_known mode=([01]) observed_accepted=0 expected_refused=true SYSTEM_before_raw=200 SYSTEM_after_raw=200 SYSTEM_violation=0 USER_violation=1 full_known_cost_supplied_ns=10000000 helper_debt_delta_ns=90000000 helper_cost_retained=1 helper_cost_paid=1 helper_births=0 actual_own_wait4=false native_process=false values_supplied=true OS_CPU_claimed=false runtime_approved=false',stdout)
    if sorted(map(int,USER_rows))!=[0,1]:raise ValueError('USER-only production API live/terminal refusals lack isolated SYSTEM-stable supplied vectors')
    report['pause_USER_only_causal_API']={'modes':{'0':'actual engine_usage path, supplied raw USER100->101/SYSTEM200 unchanged','1':'actual terminal guard, supplied5ms+1000ns boundary accepted/+1001ns refused'},'scope':'supplied production guard/known10ms CPU/receipt-ready inputs; actual production payment retains/pays90ms debt once; no native process/RI/ownwait4/OS CPU proof; physical modes9/10 remain mandatory','native_process':False,'runtime_approved':False}
    # These are genuine pre-stimulus observations from unchanged physical
    # modes9/10. Never substitute the API's supplied counters for these bytes.
    command=report['commands'][-1]
    pause_snapshots=[]
    prefix=b'private_pause_pre_stimulus_snapshot_source_known '
    for line in read_stable(command['stderr']['path'],2*1024*1024).splitlines():
        if line.startswith(prefix):pause_snapshots.append(json.loads(line[len(prefix):]))
    if sorted(x.get('mode') for x in pause_snapshots)!=[9,10]:raise ValueError('both unchanged physical pause baselines require one actual pre-stimulus snapshot')
    physical_system=[]
    for x in pause_snapshots:
        suffix=f'/total-{x["mode"]}.ndjson'
        files=[f for f in command['fixture_capture']['files'] if f['path'].endswith(suffix)]
        if len(files)!=1:raise ValueError('physical pause snapshot lacks exact retained original ledger')
        rows=[json.loads(line) for line in read_stable(files[0]['path'],2*1024*1024).splitlines()]
        h=next(v for v in rows if v.get('kind')=='helper_total_budget')
        p=next(v for v in rows if v.get('kind')=='pause_user_and_terminal_rss')
        numer,denom=p['system_timebase_numer'],p['system_timebase_denom']
        before,after=x['ACK_SYSTEM_raw'],x['current_SYSTEM_raw']
        if (not numer or not denom or before>after or x['original_pid']!=h['pid']
                or x['ACK_SYSTEM_raw']!=p['ack_system_raw'][0]
                or x['ACK_USER_raw']!=x['current_USER_raw'] or x['ACK_USER_raw']!=p['ack_user_raw'][0]
                or x['sample_return']!=1 or x['guard_result']!=1 or x['owner_failed'] or x['private_failed']
                or x['SYSTEM_violation'] or x['USER_violation'] or x['private_first_error']!='none'):
            raise ValueError('actual pre-stimulus pause lacks exact USER and admitted retained SYSTEM/source premises')
        ns=((after-before)*numer+denom-1)//denom
        if (ns>50000 or h['stopped_system_ns']<ns or h['terminal_cpu_ns']<ns
                or h['helper_debt_delta_ns']!=9*max(1000000,h['terminal_cpu_ns'])
                or not h['helper_cost_retained'] or not h['helper_cost_paid'] or not h['actual_own_wait4'] or not h['group_gone']):
            raise ValueError('actual bounded SYSTEM accrual lacks unchanged full original CPU bill and settlement')
        physical_system.append({'mode':x['mode'],'original_pid':x['original_pid'],'ACK_SYSTEM_raw':before,
            'observed_SYSTEM_raw':after,'retained_timebase_numer':numer,'retained_timebase_denom':denom,
            'stopped_SYSTEM_at_pre_stimulus_ns':ns,'nonzero_actual_SYSTEM_observed':ns>0,
            'USER_exact':True,'original_ownwait4_groupgone_full_CPU_debt_paid':True,'runtime_approved':False})
    report['physical_pause_bounded_SYSTEM_admissions']=physical_system
    stopped_api=b'private_stopped_gap_API_source_known cases=16 stopped_raw_gap_ns=130000000 stopped_running_gap_ns=20000000 exact_each_side_ns=50000000 no_ACK_refused=true running_over50_refused=true USER_growth_refused=true SYSTEM_limit_ns=50000 SYSTEM_overlimit_refused=true aggregate_SYSTEM_bound=true checked_timebase=true full_known_cost_supplied=true actual_own_wait4=false query_return_overshoot_refused=true raw_endpoints_retained=true native_process=false values_supplied=true OS_timing_claimed=false runtime_approved=false\n'
    if stdout.count(stopped_api)!=1:raise ValueError('sixteen supplied stopped/running cadence API guards missing or duplicated')
    report['stopped_cadence_API']={'cases':16,'scope':'supplied clocks/ACK/rawUSER/SYSTEM/timebase/receipt and actual production interval/payment predicates; no native signal or OS timing claim',
        'aggregate_SYSTEM_bound_ns':50000,'SYSTEM_ns_boundary':[50000,50001],'two_member_SYSTEM_ns':[[30000,20000],[30000,20001]],
        'old_raw_gap_over_refusal_control':True,'no_ACK_running_USER_SYSTEM_fresh_end_and_query_return_guards':True,'native_process':False,'runtime_approved':False}
    marker=b'private_total_retained_guard_source_known api=true supplied_clock_and_identity_values=true native_process=false runtime_approved=false\n'
    if stdout.count(marker)!=1:
        raise ValueError('current retained-clock/child production API witness missing or duplicated')
    report['retained_guard_API']='supplied scalar clock/identity preconditions; no physical orphan or child wait4 claim'
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    if b'private_quarantine_hold_source_known ' not in stdout:
        raise ValueError('current same-controller prompt/refusal/FD/owner-loss witness missing')


def delayed_ack_observation_receipt(stdout, command):
    prefix=b'private_delayed_ACK_observation_process_source_known '
    lines=[line[len(prefix):] for line in stdout.splitlines() if line.startswith(prefix)]
    if len(lines)!=1:raise ValueError('one actual delayed-ACK-observation process witness required')
    fields=dict(part.decode('ascii').split('=',1) for part in lines[0].split())
    for key in ('fixture_delays_ACK_observation','actual_native_wait_path','begin_derived_from_measured_end_minus_delta_wall','actual_GS1_GS2','actual_own_wait4_groupgone','actual_full_bill_paid','native_process'):
        if fields.get(key)!='true':raise ValueError('delayed ACK process lost required actual premise '+key)
    for key in ('OS_slow_STOP_latency_claimed','installed_tool_qualification','runtime_approved'):
        if fields.get(key)!='false':raise ValueError('delayed ACK process overclaims '+key)
    def number(key):return int(fields[key])
    begin,end,wall=(number(k) for k in ('measured_wait_begin_ns','measured_wait_return_ns','measured_wait_wall_ns'))
    sent,returned,ack,elapsed=(number(k) for k in ('STOP_call_ns','STOP_return_ns','positive_ACK_ns','STOP_to_ACK_ns'))
    if not (0<sent<=returned<=begin<end<=ack and end-begin==wall and ack-sent==elapsed<=25000000
            and number('supplied_request_cap_ns')==1000000 and number('hook_calls')==number('native_wait_calls')==1
            and number('first_paid_rest_status')==4 and number('first_paid_rest_runnable')==0):
        raise ValueError('actual bounded wait/first real ACK endpoints or raw stopped status disagree')
    completed=[line for line in stdout.splitlines() if line.startswith(b'private_same_wake_admission_completed_source_known ')]
    if len(completed)!=1:raise ValueError('actual delayed ACK process lacks original GS1/GS2/reap/bill receipt')
    bill=dict(part.decode('ascii').split('=',1) for part in completed[0].split()[1:])
    if not (int(bill['original_pid'])==number('original_pid') and int(bill['child_pid'])==number('child_pid')
            and all(bill[k]=='1' for k in ('child_image_admitted','GS1_bound','GS2_settled','actual_own_wait4','group_gone'))
            and bill['rollup_mask']=='31' and int(bill['helper_debt_delta_ns'])==9*max(1000000,int(bill['helper_CPU_ns']))
            and int(bill['debt_ns'])==int(bill['paid_ns'])):
        raise ValueError('delayed ACK original helper cost/custody/rollup or full9x bill disagrees')
    return {'scope':'fixture deliberately delays first closing ACK observation through one actual production bounded knote wait; no OS slow-stop latency claim',
            'actual':fields,'bill':bill,'stdout':command['stdout'],'fixture_capture':command['fixture_capture']}


def pause_quarantine_category_receipt(stdout, mutant=False):
    prefix=b'private_pause_quarantine_category_API_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows)!=(1 if mutant else 2):raise ValueError('central no-ownreap quarantine classifier API case count differs')
    for i,row in enumerate(rows):
        if row['case']!=str(i) or row['prior_settlement']!=str(i):raise ValueError('central classifier API case ordering differs')
        if any(row[k]!='false' for k in ('actual_quarantine_body','native_process','actual_signal','actual_ownwait4','runtime_approved')) or row['values_supplied']!='true':
            raise ValueError('central classifier API overclaims native quarantine or effects')
        if row['original_custody_retained']!='1' or any(row[k]!='0' for k in ('supplied_ownwait4','ACK_mask','resume_attempted','helper_cost_retained','helper_cost_paid')):
            raise ValueError('central classifier API lost held custody or invented reap/ACK/CPU payment')
        if mutant:
            if row['classifier_return']!='1' or row['coverage_count']!='0' or row['first_coverage']!='0' or row['first_settlement']!='1':
                raise ValueError('missing central classification mutant did not isolate old settlement-only category')
        elif row['classifier_return']!='0' or int(row['coverage_count'])<1 or int(row['settlement_count'])<1 or row['first_coverage']!=str(int(not i)) or row['first_settlement']!=str(i) or row['first_detail_preserved']!='1' or row['owner_first_preserved']!='1':
            raise ValueError('central classifier omitted coverage_unknown or overwrote FIRST error')
    return {'cases':len(rows),'rows':rows,'native_process':False,'actual_quarantine_body':False,'values_supplied':True,'scope':'production native-free central classifier plus existing later error-recording call, supplied pending original custody, no OS/quarantine loop'}


def pause_stop_exit_api_receipt(stdout, mutant=False):
    pause_quarantine_category_receipt(stdout)
    prefix=b'private_pause_stop_exit_API_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if not rows:raise ValueError('missing supplied STOP-error terminal protocol rows')
    for row in rows:
        for key in ('syscall_result_supplied','rusage_supplied','terminal_proofs_supplied'):
            if row.get(key)!='true':raise ValueError('STOP-error API omitted supplied provenance '+key)
        for key in ('actual_signal','actual_ownwait4','native_process','runtime_approved'):
            if row.get(key)!='false':raise ValueError('STOP-error API overclaims native effect '+key)
        if row['original_owned']!='1' or row['initial_ready']!='0' or row['no_group_ready']!='0' or row['ACK_mask']!='0' or row['resume_attempted']!='0':
            raise ValueError('STOP-error API minted early settlement or stop/resume proof')
    if mutant:
        row=rows[0]
        if not (row['case']=='0' and row['name']=='esrch_settled' and row['settled']=='0' and row['expected_settled']=='1'
                and row['supplied_ownwait4']==row['supplied_group_gone']=='1' and row['supplied_terminal_CPU_ns']=='2002000'
                and row['supplied_helper_debt_delta_ns']=='18018000' and row['helper_cost_retained']==row['helper_cost_paid']=='1'):
            raise ValueError('missing-terminal-path mutant did not reach exact ESRCH paid own-reap over-refusal')
        return {'scope':'supplied original identity/errno/terminal/rusage protocol; no native signal/wait4 claim','rows':rows}
    if len(rows)!=16 or [int(x['case']) for x in rows]!=list(range(16)):
        raise ValueError('STOP-error API lacks all16 bounded guard cases')
    for row in rows:
        expected=int(row['case']) in (0,5)
        if not (int(row['settled'])==int(row['expected_settled'])==int(expected) and row['ready']==row['settled']
                and int(row['terminal_resolved'])==int(expected) and row['bill_idempotent']=='1'):
            raise ValueError('STOP-error API verdict differs from original own-reap/cost premises')
    if rows[5]['zombie_only_latched']!='1' or rows[6]['zombie_only_latched']!='0' or rows[7]['zombie_only_latched']!='0':
        raise ValueError('EPERM zombie-only proof was not independently latched before cleanup')
    for index,row in enumerate(rows):
        if index==1:
            if row['helper_cost_retained']!='0':raise ValueError('no-ownwait4 vector invented a CPU receipt')
        elif int(row['supplied_helper_debt_delta_ns'])!=9*max(1000000,int(row['supplied_terminal_CPU_ns'])) and int(row['supplied_terminal_CPU_ns'])!=2**64-1:
            raise ValueError('known supplied wait4 CPU lost full9x bill')
    marker=b'private_pause_stop_exit_API_complete_source_known cases=16 passed=16 syscall_result_supplied=true rusage_supplied=true terminal_proofs_supplied=true actual_signal=false actual_ownwait4=false native_process=false runtime_approved=false\n'
    if stdout.count(marker)!=1:raise ValueError('STOP-error API incomplete exact summary')
    return {'cases':16,'rows':rows,'scope':'supplied original identity/errno/timestamps/rusage and represented wait4/group proof; native-free production predicates and real budget/payment code','actual_signal':False,'actual_ownwait4':False,'native_process':False}


def pause_preobserved_process_receipt(stdout, command):
    prefix=b'private_pause_preobserved_exit_process_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows)!=1:raise ValueError('one deterministic preobserved original-exit witness required')
    row=rows[0]
    if not (int(row['original_pid'])>0 and all(row[k]=='0' for k in ('pause_return','STOP_attempted','raw_STOP_errno','ACK_mask','resume_attempted'))
            and all(row[k]=='1' for k in ('terminal_resolved','actual_original_own_wait4','original_group_gone'))
            and 0<int(row['first_terminal_ns'])<=int(row['own_wait4_read_ns'])
            and int(row['helper_debt_delta_ns'])==9*max(1000000,int(row['terminal_CPU_ns']))
            and int(row['debt_ns'])==int(row['paid_ns']) and row['actual_kernel_ESRCH_claimed']=='false'
            and row['native_process']=='true' and row['installed_tool_qualification']==row['runtime_approved']=='false'):
        raise ValueError('preobserved-exit process lost actual original terminal/cost proof or overclaimed errno')
    return {'actual':row,'scope':'deterministic actual original exit before engine_pause; no kernel ESRCH race claimed','stdout':command['stdout'],'fixture_capture':command['fixture_capture']}


def verify_pause_stop_exit(directory, inputs, report):
    source=inputs[str(SOURCE)];current=directory/'current';current.mkdir(mode=0o700)
    for name in (SOURCE,CONTROLS):report['snapshots'].append(retain(current/name.name,inputs[str(name)]))
    program=compile_source(current,CONTROLS.name,report)
    code,stdout,_=run([str(program),'pause-stop-exit-api'],current,'pause-stop-exit-api',report,fixture=True)
    expect(code,0,'supplied failed STOP/own-reap terminal protocol')
    report['pause_stop_exit_API']=pause_stop_exit_api_receipt(stdout)
    code,stdout,_=run([str(program),'pause-stop-exit-control'],current,'preobserved-exit-process',report,fixture=True)
    expect(code,0,'actual original exit before pause and original own-reap bill')
    report['pause_preobserved_exit_process']=pause_preobserved_process_receipt(stdout,report['commands'][-1])
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    report['causal_mutants']=[]
    for name in ('pause-stop-exit-ignore-path','pause-stop-no-reap-category'):
        changed_source,args=variants(source)[name];own=directory/name;own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own/SOURCE.name,changed_source));report['snapshots'].append(retain(own/CONTROLS.name,inputs[str(CONTROLS)]))
        program=compile_source(own,CONTROLS.name,report)
        code,stdout,stderr=run([str(program),*args],own,'controls',report,fixture=True)
        expect(code,81,'causal missing terminal-before-stop path/category '+name)
        if ASSERTIONS[name] not in stderr:raise ValueError('terminal-before-stop mutant missed exact assertion '+name)
        if name=='pause-stop-exit-ignore-path':report['ignored_path_API']=pause_stop_exit_api_receipt(stdout,mutant=True)
        else:report['no_reap_category_API_mutant']=pause_quarantine_category_receipt(stdout,mutant=True)
        report['causal_mutants'].append({'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),'native_process':False,'values_supplied':True})
    report['source_mutants']=2;report['canonical_suite_receipt']=False



def observation_resume_terminal_receipt(stdout, wave, *, mutant=False):
    prefix=b'private_observation_resume_terminal_process_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows)!=1:raise ValueError('one actual resume terminal wave receipt required')
    row=rows[0]
    if (int(row['wave'])!=wave or int(row['opening_samples'])!=wave or row['resume_return']!='0'
            or row['selected_boundary_hits']!='1' or row['seen_site']!='1' or row['takeovers']!='1' or row['terminal_pending']!='1'
            or row['owner_failed_before_retire']!=str(int(mutant)) or bool(int(row['coverage_unknown_before_retire']))!=mutant
            or row['actual_ACK_mask']!='1' or row['resume_attempted']!='0' or row['resume_admitted']!='0'
            or row['existing_group_kill_effects']!='1' or int(row['actual_kill_cut_ns'])<=0
            or int(row['terminal_start'])<=0 or int(row['terminal_exit'])<=0):
        raise ValueError('resume terminal wave was promoted into live failure or resumed work')
    for key in ('actual_own_wait4','group_gone','retired_coverage','helper_cost_retained','helper_cost_paid'):
        if row[key]!='1':raise ValueError('resume wave lacks actual original physical/full cost closure '+key)
    for key in ('syscall_outcome_supplied','currentness_premise_supplied','actual_positive_ACK','actual_SIGKILL','native_process'):
        if row[key]!='true':raise ValueError('resume wave omits actual/supplied scope '+key)
    for key in ('target_syscall_invoked','paused_phase_supplied','spontaneous_exit_claimed','installed_tool_qualification','runtime_approved'):
        if row[key]!='false':raise ValueError('resume wave overclaims '+key)
    if row['debt_ns']!=row['paid_ns'] or int(row['helper_debt_ns'])!=9*max(1000000,int(row['CPU_ns'])):
        raise ValueError('resume wave loses full actual CPU debt')
    return row


def observation_event_interrupt_receipt(stdout, cancelled, *, mutant=False):
    prefix=b'private_observation_event_interrupt_process_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows)!=1:raise ValueError('one supplied EINTR/cancellation original-owner receipt required')
    row=rows[0]
    if (int(row['cancelled_premise'])!=cancelled or row['event_return']!=str(int(mutant))
            or row['selected_boundary_hits']!='1' or row['seen_site']!='22' or int(row['raw_errno'])!=errno.EINTR
            or bool(int(row['coverage_unknown']))==mutant
            or row['first_cancelled']!=str(int(bool(cancelled) and not mutant))
            or row['first_coverage']!=str(int(not cancelled and not mutant))):
        raise ValueError('event EINTR was hidden or overwrote an already-observed FIRST cancellation')
    if any(row[k]!='1' for k in ('actual_own_wait4','group_gone','helper_cost_retained','helper_cost_paid')):
        raise ValueError('EINTR fixture lacks actual original effect/cost settlement')
    if any(row[k]!='true' for k in ('syscall_outcome_supplied','cancellation_state_supplied','native_process')) or any(row[k]!='false' for k in ('target_syscall_invoked','actual_signal_interruption','runtime_approved')):
        raise ValueError('EINTR fixture overclaims native signal timing')
    if row['debt_ns']!=row['paid_ns'] or int(row['helper_debt_ns'])!=9*max(1000000,int(row['CPU_ns'])):
        raise ValueError('EINTR fixture loses full actual CPU debt')
    return row


def observation_mutant_receipt(stdout,args,name):
    if args[0]=='terminal-both-policy-api':return terminal_both_policy_api_receipt(stdout,mutant=True)
    if args[0]=='total-control' and args[1]=='7':return observation_kill_compat_receipt(stdout,mutant=True)
    if args[0]=='observation-resume-control':return observation_resume_terminal_receipt(stdout,int(args[1]),mutant=True)
    if args[0]=='observation-event-interrupt-control':return observation_event_interrupt_receipt(stdout,int(args[1]),mutant=True)
    return observation_race_receipt(stdout,int(args[1]),int(args[2]),mutant=True,fatal_guard=name=='observation-group-census-fatal')


def observation_kill_compat_receipt(stdout, *, mutant=False):
    prefix=b'private_total_property_axis_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows)!=1:raise ValueError('one unchanged old mode7 physical/cost prerequisite axis required')
    row=rows[0]
    if (row['mode']!='7' or row['own_wait4']!='1' or row['group_gone']!='1' or row['terminal_pass']!='1'
            or int(row['terminal_cpu_ns'])!=int(row['authoritative_cpu_ns']) or int(row['terminal_cpu_ns'])>=2**64-1
            or row['runtime_approved']!='false'):
        raise ValueError('old mode7 lost original physical/full-known-cost prerequisites')
    prefix=b'private_total_budget_control_source_known '
    complete=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if mutant:
        if complete:raise ValueError('unconditional skip mutant unexpectedly passed old natural-exit assertion')
        return {'actual_axis':row,'scope':'unchanged old source asserts actual native/cost/ownreap/groupgone prerequisites before this axis; named raced-exit assertion then fails',
                'old_named_assertion_reached':True,'runtime_approved':False}
    if len(complete)!=1:raise ValueError('old mode7 positive lacks final pending/raced-exit bill receipt')
    bill=complete[0]
    if (bill['mode']!='7' or bill['actual_own_wait4']!='1' or bill['group_gone']!='1' or bill['raced_exit']!='1'
            or bill['helper_cost_retained']!='1' or bill['helper_cost_paid']!='1'
            or int(bill['terminal_cpu_ns'])!=int(row['terminal_cpu_ns']) or bill['debt_ns']!=bill['paid_ns']):
        raise ValueError('normal preobserved kill lost its unchanged original raced-exit settlement')
    return {'actual_axis':row,'bill':bill,'scope':'existing physical original-ownwait4/fullbill witness; final raw EPERM source-known seam, not claimed kernel EPERM timing',
            'runtime_approved':False}


def terminal_both_policy_api_receipt(stdout, *, mutant=False):
    prefix=b'private_terminal_both_policy_API_source_known '
    rows=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows)!=(1 if mutant else 2):raise ValueError('both-policy API must retain each supplied prior-FIRST case')
    for i,row in enumerate(rows):
        if (int(row['case'])!=i or row['prior_FIRST_supplied']!=('true' if i else 'false')
                or row['terminal_identity']!='1' or row['live_admitted']!='0' or row['terminal_user_accepted']!='0'
                or row['USER_violation']!='1' or row['SYSTEM_violation']!='1'
                or row['terminal_USER_checked']!=str(int(not mutant))
                or row['terminal_USER_ns']!=('0' if mutant else '1101') or row['baseline_USER_ns']!='100'
                or row['terminal_delta_expected_ns']!='1001' or row['SYSTEM_growth_ns']!='50001'
                or row['first_preserved']!='1' or int(row['prior_source_first'])!=i):
            raise ValueError('SYSTEM refusal suppressed independent terminal USER diagnosis or changed FIRST')
        if not mutant and int(row['USER_excess_ns'])<1001:raise ValueError('terminal USER excess was not retained')
        for key in ('raw_RI_supplied','raw_timebase_supplied','terminal_USER_supplied'):
            if row[key]!='true':raise ValueError('both-policy API missing supplied provenance '+key)
        for key in ('actual_native_RI','actual_own_wait4','native_process','runtime_approved'):
            if row[key]!='false':raise ValueError('both-policy API overclaims native effect '+key)
        if (row['supplied_terminal_CPU_ns']!='10000000' or row['helper_debt_ns']!='90000000'
                or row['helper_cost_retained']!='1' or row['helper_cost_paid']!='1' or row['debt_ns']!=row['paid_ns']):
            raise ValueError('both-policy API dropped or duplicated full supplied known CPU bill')
    return {'rows':rows,'scope':'supplied positive RI-exit/start/counters and terminal USER/CPU; production identity/policy/diagnosis/payment functions, no native RI/ownwait4/OS CPU claim'}


def terminal_policy_physical_receipt(stdout, command):
    prefix=b'private_total_budget_control_source_known '
    bills=[dict(x.decode('ascii').split('=',1) for x in line[len(prefix):].split()) for line in stdout.splitlines() if line.startswith(prefix)]
    if len(bills)!=1:raise ValueError('unchanged actual mode9 lacks final original bill receipt')
    bill=bills[0]
    if (bill['mode']!='9' or bill['actual_own_wait4']!='1' or bill['group_gone']!='1'
            or bill['helper_cost_retained']!='1' or bill['helper_cost_paid']!='1'
            or int(bill['terminal_cpu_ns'])>=2**64-1 or bill['debt_ns']!=bill['paid_ns']):
        raise ValueError('actual mode9 lost original physical/full CPU closure')
    files=[x for x in command['fixture_capture']['files'] if x['path'].endswith('/total-9.ndjson')]
    if len(files)!=1:raise ValueError('mode9 lacks same-owned retained ledger')
    rows=[json.loads(x) for x in read_stable(files[0]['path'],2*1024*1024).splitlines()]
    pauses=[x for x in rows if x.get('kind')=='pause_user_and_terminal_rss']
    if not pauses:raise ValueError('actual mode9 lacks terminal independent USER diagnostics')
    pause=pauses[-1]
    if (pause['terminal_checked']!=1 or pause['sticky_violation']!=1 or pause['pause_user_pass']!=0
            or pause['resume_admitted']!=0 or pause['terminal_user_ns']<=pause['baseline_user_ns']+1000):
        raise ValueError('actual mode9 skipped terminal USER diagnosis or admitted burned USER')
    return {'bill':bill,'pause':pause,'ledger':files[0],
            'scope':'unchanged real external-CONT terminal USER mode9; SYSTEM may also refuse; no new physical timing premise'}


def observation_prekill_retirement_receipt(stdout):
    prefix = b'private_observation_prekill_retirement_process_source_known '
    rows = [dict(part.decode('ascii').split('=', 1) for part in line[len(prefix):].split())
            for line in stdout.splitlines() if line.startswith(prefix)]
    if len(rows) != 1:
        raise ValueError('one same-retirement prekill terminal process receipt required')
    row = rows[0]
    for field in ('initial_original_live', 'syscall_outcome_supplied', 'native_process'):
        if row[field] != 'true':
            raise ValueError('whole prekill receipt omitted actual/supplied provenance: ' + field)
    for field in ('target_syscall_invoked', 'terminal_proof_supplied', 'installed_tool_qualification', 'runtime_approved'):
        if row[field] != 'false':
            raise ValueError('whole prekill receipt overclaims native error or qualification: ' + field)
    if (row['site'] != 'prekill_bsd' or row['site_id'] != '17' or row['retirement_calls'] != '1'
            or row['selected_boundary_hits'] != '1' or row['seen_site'] != '17' or row['takeovers'] != '1'
            or int(row['raw_errno']) != errno.ESRCH or int(row['original_pid']) <= 0
            or int(row['terminal_start']) <= 0 or int(row['terminal_exit']) <= 0
            or any(row[k] != '1' for k in ('actual_own_wait4', 'group_gone', 'retired_coverage', 'helper_cost_retained', 'helper_cost_paid'))
            or any(row[k] != '0' for k in ('quarantine', 'ACK_mask', 'resume_admitted'))
            or int(row['CPU_ns']) >= 2**64 - 1 or int(row['helper_debt_ns']) != 9 * max(1000000, int(row['CPU_ns']))
            or row['debt_ns'] != row['paid_ns']):
        raise ValueError('prekill takeover did not finish original custody/cost in its SAME retirement invocation')
    return row


def observation_baselines(program, current, report):
    report['observation_production_bindings'] = observation_production_bindings(read_stable(current / SOURCE.name, 1024 * 1024))
    code, stdout, _ = run([str(program), 'observation-predicate-api'], current, 'observation-predicate-api', report, fixture=True)
    expect(code, 0, 'fixed observation identity and raw-domain API')
    marker = (b'private_observation_predicate_API_source_known sites=27 identity_cases=10 raw_domain_whitelist=true '
              b'foreign_birth_start_full_token_refused=true reaped_unowned_duplicate_missing_row_refused=true '
              b'native_process=false values_supplied=true actual_terminal_proof=false actual_own_wait4=false runtime_approved=false\n')
    if stdout.count(marker) != 1:
        raise ValueError('all fixed observation domain/identity predicates require explicit supplied scope')
    rows = []
    for number, site in enumerate(OBSERVATION_SITES, 1):
        code, stdout, _ = run([str(program), 'observation-race-control', str(number), '0'], current,
                              'observation-positive-' + site, report, fixture=True)
        expect(code, 0, 'selected production observation terminal path ' + site)
        rows.append(observation_race_receipt(stdout, number))
        report['process_ledgers'] += report['commands'][-1]['fixture_capture']['files']
    negatives = []
    for case in range(1, 7):
        code, stdout, stderr = run([str(program), 'observation-race-control', '1', str(case)], current,
                                   'observation-refusal-' + str(case), report, fixture=True)
        expect(code, 81, 'other error/foreign identity/missing terminal proof observation ' + str(case))
        negatives.append(observation_race_receipt(stdout, 1, case))
        if b'probe refused:' in stderr:
            raise ValueError('observation refusal failed a fixture premise instead of its completed protocol')
        report['process_ledgers'] += report['commands'][-1]['fixture_capture']['files']
    code, stdout, _ = run([str(program), 'observation-race-control', '1', '7'], current, 'observation-advisory-notready', report, fixture=True)
    expect(code, 0, 'independent terminal RI proof is sufficient despite supplied advisory waitability zero')
    report['observation_advisory_notready'] = observation_race_receipt(stdout, 1, 7)
    report['process_ledgers'] += report['commands'][-1]['fixture_capture']['files']
    code, stdout, _ = run([str(program), 'observation-prekill-control'], current, 'observation-prekill-whole-retirement', report, fixture=True)
    expect(code, 0, 'actual prekill terminal takeover through same original retirement invocation')
    report['observation_prekill_whole_retirement'] = observation_prekill_retirement_receipt(stdout)
    report['process_ledgers'] += report['commands'][-1]['fixture_capture']['files']
    report['observation_resume_terminal_waves']=[]
    for wave in (1,2):
        code, stdout, _ = run([str(program), 'observation-resume-control', str(wave)], current, 'observation-resume-wave-'+str(wave), report, fixture=True)
        expect(code, 0, 'whole resume terminal sample wave '+str(wave))
        report['observation_resume_terminal_waves'].append(observation_resume_terminal_receipt(stdout,wave))
        report['process_ledgers'] += report['commands'][-1]['fixture_capture']['files']
    report['observation_event_interrupts']=[]
    for cancelled in (0,1):
        code, stdout, _ = run([str(program), 'observation-event-interrupt-control', str(cancelled)], current, 'observation-event-EINTR-'+str(cancelled), report, fixture=True)
        expect(code, 0, 'supplied bare/cancelled event EINTR '+str(cancelled))
        report['observation_event_interrupts'].append(observation_event_interrupt_receipt(stdout,cancelled))
        report['process_ledgers'] += report['commands'][-1]['fixture_capture']['files']
    code, stdout, _ = run([str(program), 'terminal-both-policy-api'], current, 'terminal-both-policy-api', report, fixture=True)
    expect(code, 0, 'supplied independent terminal USER and SYSTEM policy diagnostics')
    report['terminal_both_policy_API']=terminal_both_policy_api_receipt(stdout)
    report['observation_terminal_matrix'] = {
        'fixed_sites': list(OBSERVATION_SITES), 'positive_receipts': rows, 'negative_receipts': negatives,
        'negative_cases': {'1':'other native error','2':'supplied foreign birth','3':'supplied FULL token difference','4':'actual original still live, missing independent terminal proof','5':'supplied already-reaped flag on still-owned fixture','6':'supplied missing original custody flag'},
        'baseline_commands': 41, 'raw_error_injected': True,
        'scope': 'selected production raw boundary and caller wiring; each receipt distinguishes actual original settlement from supplied phase/proof inputs',
        'runtime_approved': False}



def verify_observation_selected_route(directory, inputs, report):
    # One changed selected-route method: preserve strict outer cleanup while
    # measuring the actual fixed-site disposition before that cleanup can run.
    source=inputs[str(SOURCE)]
    current=directory/'current';current.mkdir(mode=0o700)
    for name in (SOURCE,CONTROLS):report['snapshots'].append(retain(current/name.name,inputs[str(name)]))
    program=compile_source(current,CONTROLS.name,report)
    report['observation_production_bindings']=observation_production_bindings(source)
    code,stdout,_=run([str(program),'observation-race-control','8','0'],current,'selected-live-rusage',report,fixture=True)
    expect(code,0,'selected live_rusage qualified terminal route and whole-caller success')
    report['selected_route_positive']=observation_race_receipt(stdout,8)
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    name='observation-bypass-live_rusage';changed_source,args=observation_variants(source)[name]
    own=directory/name;own.mkdir(mode=0o700)
    report['snapshots'].append(retain(own/SOURCE.name,changed_source));report['snapshots'].append(retain(own/CONTROLS.name,inputs[str(CONTROLS)]))
    mutant=compile_source(own,CONTROLS.name,report)
    code,stdout,stderr=run([str(mutant),*args],own,'controls',report,fixture=True)
    expect(code,81,'selected live_rusage bypass refuses at its exact production route')
    if ASSERTIONS[name] not in stderr:raise ValueError('selected bypass missed its original intended terminal-disposition assertion')
    report['causal_mutants']=[{'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),
        'receipt':observation_race_receipt(stdout,8,mutant=True),
        'scope':'actual selected route-return snapshot; later independent strict cleanup remains separately visible'}]
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    report['baseline_commands']=1;report['source_mutants']=1;report['canonical_suite_receipt']=False


def verify_terminal_policy(directory, inputs, report):
    source=inputs[str(SOURCE)]
    current=directory/'current';current.mkdir(mode=0o700)
    for name in (SOURCE,CONTROLS):report['snapshots'].append(retain(current/name.name,inputs[str(name)]))
    program=compile_source(current,CONTROLS.name,report)
    code,stdout,_=run([str(program),'terminal-both-policy-api'],current,'both-policy-api',report,fixture=True)
    expect(code,0,'supplied BOTH policy axes and prior FIRST diagnosis')
    report['terminal_both_policy_API']=terminal_both_policy_api_receipt(stdout)
    code,stdout,_=run([str(program),'total-control','9'],current,'unchanged-physical-mode9',report,fixture=True)
    expect(code,0,'unchanged actual terminal USER mode9')
    report['actual_terminal_USER_mode9']=terminal_policy_physical_receipt(stdout,report['commands'][-1])
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    name='observation-terminal-user-sys-early';changed_source,args=observation_variants(source)[name]
    own=directory/name;own.mkdir(mode=0o700)
    report['snapshots'].append(retain(own/SOURCE.name,changed_source));report['snapshots'].append(retain(own/CONTROLS.name,inputs[str(CONTROLS)]))
    mutant=compile_source(own,CONTROLS.name,report)
    code,stdout,stderr=run([str(mutant),*args],own,'controls',report,fixture=True)
    expect(code,81,'restored SYSTEM-first terminal USER diagnostic omission')
    if ASSERTIONS[name] not in stderr:raise ValueError('terminal USER omission missed its independent-diagnostic assertion')
    report['causal_mutants']=[{'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),
                              'receipt':terminal_both_policy_api_receipt(stdout,mutant=True),'native_process':False}]
    report['source_mutants']=1;report['canonical_suite_receipt']=False


def verify_observation_kill_compat(directory, inputs, report):
    source=inputs[str(SOURCE)]
    current=directory/'current';current.mkdir(mode=0o700)
    for name in (SOURCE,CONTROLS):report['snapshots'].append(retain(current/name.name,inputs[str(name)]))
    program=compile_source(current,CONTROLS.name,report)
    code,stdout,_=run([str(program),'total-control','7'],current,'old-natural-exit-mode7',report,fixture=True)
    expect(code,0,'unchanged preobserved original final-kill raced-exit witness')
    report['legacy_mode7']=observation_kill_compat_receipt(stdout)
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    code,stdout,_=run([str(program),'observation-prekill-control'],current,'new-prekill-takeover',report,fixture=True)
    expect(code,0,'new PREKILL takeover same-retirement witness')
    report['observation_prekill_whole_retirement']=observation_prekill_retirement_receipt(stdout)
    report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    name='observation-normal-terminal-kill-skip';changed_source,args=observation_variants(source)[name]
    own=directory/name;own.mkdir(mode=0o700)
    report['snapshots'].append(retain(own/SOURCE.name,changed_source));report['snapshots'].append(retain(own/CONTROLS.name,inputs[str(CONTROLS)]))
    mutant=compile_source(own,CONTROLS.name,report)
    code,stdout,stderr=run([str(mutant),*args],own,'controls',report,fixture=True)
    expect(code,81,'restored unconditional terminal-group kill skip')
    if ASSERTIONS[name] not in stderr:raise ValueError('unconditional kill-skip mutant missed unchanged old assertion')
    report['causal_mutants']=[{'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),
                              'receipt':observation_kill_compat_receipt(stdout,mutant=True)}]
    report['source_mutants']=1;report['canonical_suite_receipt']=False


def verify_observation_outer_tail(directory, inputs, report):
    # Changed mutant methodology only: reuse the four affected whole-caller
    # baselines and three exact old-policy restorations, without replaying the
    # already-observed27 independent raw-boundary process positives.
    source=inputs[str(SOURCE)]
    current=directory/'current';current.mkdir(mode=0o700)
    for name in (SOURCE,CONTROLS):report['snapshots'].append(retain(current/name.name,inputs[str(name)]))
    program=compile_source(current,CONTROLS.name,report)
    report['observation_production_bindings']=observation_production_bindings(source)
    report['outer_resume_waves']=[];report['outer_event_interrupts']=[]
    for wave in (1,2):
        code,stdout,_=run([str(program),'observation-resume-control',str(wave)],current,'resume-wave-'+str(wave),report,fixture=True)
        expect(code,0,'whole terminal resume wave '+str(wave))
        report['outer_resume_waves'].append(observation_resume_terminal_receipt(stdout,wave))
        report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    for cancelled in (0,1):
        code,stdout,_=run([str(program),'observation-event-interrupt-control',str(cancelled)],current,'event-EINTR-'+str(cancelled),report,fixture=True)
        expect(code,0,'event EINTR/cancelled premise '+str(cancelled))
        report['outer_event_interrupts'].append(observation_event_interrupt_receipt(stdout,cancelled))
        report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
    variants=observation_variants(source)
    names=('observation-resume-first-fatal','observation-resume-fresh-fatal','observation-event-eintr-success')
    report['causal_mutants']=[]
    for name in names:
        changed_source,args=variants[name];own=directory/name;own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own/SOURCE.name,changed_source));report['snapshots'].append(retain(own/CONTROLS.name,inputs[str(CONTROLS)]))
        mutant=compile_source(own,CONTROLS.name,report)
        code,stdout,stderr=run([str(mutant),*args],own,'controls',report,fixture=True)
        expect(code,81,'exact old outer policy '+name)
        if ASSERTIONS[name] not in stderr:raise ValueError('outer policy mutant missed intended assertion '+name)
        row=observation_mutant_receipt(stdout,args,name)
        report['causal_mutants'].append({'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),'receipt':row,
                                        'scope':'exact old compound terminal-fatal or EINTR-success policy, actual/supplied facts distinguished'})
    report['source_mutants']=len(names);report['canonical_suite_receipt']=False


def verify_observation_races(directory, inputs, report):
    source = inputs[str(SOURCE)]
    current = directory / 'current';current.mkdir(mode=0o700)
    for name in (SOURCE, CONTROLS):
        report['snapshots'].append(retain(current / name.name, inputs[str(name)]))
    program = compile_source(current, CONTROLS.name, report)
    observation_baselines(program, current, report)
    report['causal_mutants'] = []
    mutations=observation_variants(source)
    for name, (changed_source, args) in mutations.items():
        own = directory / name;own.mkdir(mode=0o700)
        report['snapshots'].append(retain(own / SOURCE.name, changed_source))
        report['snapshots'].append(retain(own / CONTROLS.name, inputs[str(CONTROLS)]))
        mutant = compile_source(own, CONTROLS.name, report)
        code, stdout, stderr = run([str(mutant), *args], own, 'controls', report, fixture=True)
        expect(code, 81, 'bypassed selected production observation route ' + name)
        if ASSERTIONS[name] not in stderr:
            raise ValueError('observation mutation missed its selected caller assertion: ' + name)
        row = observation_mutant_receipt(stdout,args,name)
        report['causal_mutants'].append({'name': name, 'exit_code': code, 'required_assertion': ASSERTIONS[name].decode(),
                                        'receipt': row, 'scope': 'fixed production route bypass or named outer fatal-guard/error-classification restoration; actual/supplied facts remain explicit in receipt'})
    report['source_mutants'] = len(mutations)
    report['canonical_suite_receipt'] = False


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
            if mode==5:report['stopped_cadence_actual_process']=stopped_cadence_receipt(stdout)
        code,stdout,_=run([str(program),'total-retained-guard-api'],current,'retained-guard-api',report,fixture=True)
        expect(code,0,'current retained-clock/child production API')
        code,stdout,_=run([str(program),'stopped-gap-api'],current,'stopped-gap-api',report,fixture=True)
        expect(code,0,'current stopped/running cadence production API')
    else:
        code,stdout,stderr=run([str(program)],current,'controls',report,fixture=True)
        expect(code,0,'current remaining source-known controls')
        retain_current(stdout,report)
        observation_baselines(program,current,report)
        code,pause_stdout,_=run([str(program),'pause-stop-exit-control'],current,'preobserved-exit-process',report,fixture=True)
        expect(code,0,'actual original exit before foreground pause')
        report['pause_preobserved_exit_process']=pause_preobserved_process_receipt(pause_stdout,report['commands'][-1])
        report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
        code,delayed_stdout,_=run([str(program),'same-wake-delayed-ack-control'],current,'delayed-ACK-observation-positive',report,fixture=True)
        expect(code,0,'actual first-ACK-observation delay through production bounded retained-knote wait')
        report['delayed_ACK_observation_process']=delayed_ack_observation_receipt(delayed_stdout,report['commands'][-1])
        report['process_ledgers']+=report['commands'][-1]['fixture_capture']['files']
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
    if not total_only:all_variants.update(observation_variants(source))
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
        if name in ('total-pause-live-user','total-pause-terminal-user'):
            mode=0 if name=='total-pause-live-user' else 1
            marker=(f'private_pause_USER_only_API_source_known mode={mode} observed_accepted=1 expected_refused=true SYSTEM_before_raw=200 SYSTEM_after_raw=200 SYSTEM_violation=0 USER_violation=0 full_known_cost_supplied_ns=10000000 helper_debt_delta_ns=90000000 helper_cost_retained=1 helper_cost_paid=1 helper_births=0 actual_own_wait4=false native_process=false values_supplied=true OS_CPU_claimed=false runtime_approved=false').encode()
            if marker not in stdout:raise ValueError('USER mutant did not reach isolated wrong acceptance with unchanged SYSTEM')
        if name.startswith('observation-'):
            report['commands'][-1]['observation_terminal_receipt']=observation_mutant_receipt(stdout,args,name)
        if name=='member-peak-drop-admission':
            marker=b'private_members_peak_admission_API_source_known cases=1 retained_live_count=1 members_max=1 exact_current_rows_supplied=2 final_conjunction_passed=false first_failing_term="members_max_complete" final_error_code=coverage_unknown other_final_terms_supplied=true helper_births=0 native_process=false values_supplied=true actual_GS1=false actual_GS2=false actual_own_wait4=false actual_group_gone=false actual_helper_bill=false OS_timing_claimed=false full15_physical_run_claimed=false runtime_approved=false\n'
            if marker not in stdout:
                raise ValueError('missing-admission-peak API mutant missed exact supplied current-member count/classifier cause')
        if name=='pause-stop-exit-ignore-path':report['ignored_path_API']=pause_stop_exit_api_receipt(stdout,mutant=True)
        if name=='pause-stop-no-reap-category':report['no_reap_category_API_mutant']=pause_quarantine_category_receipt(stdout,mutant=True)
        if name=='custody-stop-ignore-current-RI':
            if b'private_custody_STOP_identity_API_source_known cases=7 accepted_mask=127 expected_mask=1' not in stdout:
                raise ValueError('custody-STOP identity mutant missed its fresh-RI/failed-admission safety assertion')
        if name=='same-wake-nonblocking-only':
            marker=b'private_same_wake_event_wait_API_source_known admitted=0 supplied_CPU_ns=400000 supplied_elapsed_ns=100000 supplied_steps=10 supplied_rollup_mask=30 sticky_child_cost_unproven=1 native_process=false values_supplied=true OS_CPU_claimed=false runtime_approved=false\n'
            if marker not in stdout:
                raise ValueError('nonblocking-only API mutant missed unchanged400us-headroom refusal cause')
        if name=='same-wake-post-stop-nonblocking-only':
            marker=b'private_stop_ACK_API_source_known case=delayed_ACK admitted=0 supplied_CPU_ns=400000 supplied_STOP_to_ACK_ns=100100 supplied_steps=10 sticky_child_cost_unproven=1 native_process=false values_supplied=true actual_STOP=false actual_native_wait=false actual_own_wait4=false OS_latency_claimed=false runtime_approved=false\n'
            if marker not in stdout:raise ValueError('post-STOP zero-timeout API mutant missed unchanged400us refusal before suppliedACK')
        if name=='same-wake-insert-pre-stop-rest':
            marker=b'private_same_wake_admission_API_source_known captured=1 closed_paid_unit_before_ACK=1 supplied_stopped_ACK=0 supplied_GS1=0 supplied_rollup_mask=30 sticky_child_cost_unproven=1 native_process=false values_supplied=true actual_GS1=false actual_own_wait4=false runtime_approved=false\n'
            if marker not in stdout:
                raise ValueError('pre-STOP-rest API mutant missed exact unpaid-window loss/bit1/latch cause')
        if name in ('verifier-poll-only','verifier-skip-terminal-drain'):
            report['commands'][-1]['actual_kernel_child_lifetime']=verifier_event_lifetime(stdout)
        if name=='total-qos' and (b'detail=live helper priority differs from observed background band4' not in stderr
                or b'actual_reaped=1 group_gone=1' not in stderr):
            raise ValueError('missing-QoS mutation refused for another reason or lacks own cleanup')
        if name in ('quarantine-exit','quarantine-fd-release','hold-unpaid-observation'):
            verify_lifetime_mutant_cleanup(stderr,report['commands'][-1])
        if name=='quarantine-exit':
            verify_controller_liveness_mutant(stderr,report['commands'][-1])
        scope='source-known API/owned fixture, never installed Git/GPG qualification'
        if name in ('total-pause-live-user','total-pause-terminal-user'):
            scope='supplied USER-only production API; SYSTEM fixed; no native process/RI/ownwait4/OS CPU claim'
        if name=='member-peak-drop-admission':
            scope='supplied exact-current live member admission/count/final-classifier API; no native process, GS1, reap, OS timing or helper-bill claim'
        if name=='same-wake-post-stop-nonblocking-only':
            scope='supplied production post-STOP wait/bound API; no actual STOP/native wait/ownwait4/OS latency claim'
        item={'name':name,'exit_code':code,'required_assertion':ASSERTIONS[name].decode(),'scope':scope}
        if name in ('total-pause-live-user','total-pause-terminal-user'):item.update(native_process=False,values_supplied=True,actual_own_wait4=False)
        if name=='member-peak-drop-admission':item.update(native_process=False,values_supplied=True,actual_GS1=False,actual_own_wait4=False,actual_helper_bill=False)
        if name=='same-wake-post-stop-nonblocking-only':item.update(native_process=False,values_supplied=True,actual_STOP=False,actual_native_wait=False,actual_own_wait4=False,OS_latency_claimed=False)
        if name=='pause-stop-exit-ignore-path':item.update(native_process=False,values_supplied=True,actual_signal=False,actual_ownwait4=False,scope='supplied failed STOP/own-reap protocol; native-free predicates, no physical ESRCH claim')
        if name=='pause-stop-no-reap-category':item.update(native_process=False,values_supplied=True,actual_quarantine_body=False,scope='native-free central category; supplied pending custody, no native quarantine or ownreap')
        report.setdefault('causal_mutants',[]).append(item)
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
    parser.add_argument('--pause-stop-exit-only',action='store_true',help='affected supplied STOP-error/terminal protocol, deterministic preobserved-exit process and missing-path mutant only; not full C receipt')
    parser.add_argument('--observation-selected-route-only',action='store_true',help='selected live_rusage route-return positive and exact bypass with original full settlement; not full C receipt')
    parser.add_argument('--observation-races-only',action='store_true',help='fixed27 production observation callsites, eligibility/domain negatives and per-callsite bypass mutants only; not full C receipt')
    parser.add_argument('--observation-outer-tail-only',action='store_true',help='whole resume two waves and bare/cancelled EINTR with their three exact old-policy mutants; not full C receipt')
    parser.add_argument('--observation-kill-compat-only',action='store_true',help='unchanged old natural-exit mode7, new whole PREKILL takeover, and unconditional-skip mutant; not full C receipt')
    parser.add_argument('--terminal-policy-only',action='store_true',help='supplied BOTH-axis/prior-FIRST API, unchanged actual mode9 and SYSTEM-early diagnostic mutant; not full C receipt')
    args=parser.parse_args();target=Path(args.target_dir)
    if sum((args.total_budget_only,args.pause_stop_exit_only,args.observation_races_only,args.observation_outer_tail_only,args.observation_kill_compat_only,args.terminal_policy_only,args.observation_selected_route_only))>1:parser.error('choose only one supplemental control focus')
    if not target.is_absolute():parser.error('--target-dir must be absolute')
    target.mkdir(parents=True,exist_ok=True)
    directory=Path(tempfile.mkdtemp(prefix='private-engine-observation-selected-' if args.observation_selected_route_only else 'private-engine-terminal-policy-' if args.terminal_policy_only else 'private-engine-observation-kill-compat-' if args.observation_kill_compat_only else 'private-engine-observation-outer-' if args.observation_outer_tail_only else 'private-engine-observation-races-' if args.observation_races_only else 'private-engine-pause-stop-exit-' if args.pause_stop_exit_only else 'private-engine-total-budget-' if args.total_budget_only else 'private-engine-controls-',dir=target)).resolve()
    report={'format':'bboxy.dream.private-engine-source-controls/1','decision':66280,'scope':'supplemental_selected_observation_route' if args.observation_selected_route_only else 'supplemental_terminal_policy_diagnosis' if args.terminal_policy_only else 'supplemental_observation_kill_compatibility' if args.observation_kill_compat_only else 'supplemental_observation_outer_tail' if args.observation_outer_tail_only else 'supplemental_observation_terminal_races' if args.observation_races_only else 'supplemental_pause_stop_exit_race' if args.pause_stop_exit_only else 'supplemental_total_budget' if args.total_budget_only else 'canonical_remaining_C_controls','synthetic_source_known':True,'real_candidate_created':0,'runtime_approved':False,'whole_command_lifetime_proof':False,'live_kernel_custody_after_probe_exit_proved':False,'recovery_handoff_proved':False,'same_controller_quarantine_hold_witness_required':not args.pause_stop_exit_only,'commands':[],'inputs':{},'snapshots':[],'executables':[],'process_ledgers':[]}
    status=1
    try:
        inputs={str(p):read_stable(ROOT/p,1024*1024) for p in (SOURCE,CONTROLS,DRIVER)}
        archive=directory/'inputs';archive.mkdir(mode=0o700)
        for name,data in inputs.items():report['inputs'][name]=retain(archive/Path(name).name,data)
        if args.observation_selected_route_only:verify_observation_selected_route(directory,inputs,report)
        elif args.terminal_policy_only:verify_terminal_policy(directory,inputs,report)
        elif args.observation_kill_compat_only:verify_observation_kill_compat(directory,inputs,report)
        elif args.observation_outer_tail_only:verify_observation_outer_tail(directory,inputs,report)
        elif args.observation_races_only:verify_observation_races(directory,inputs,report)
        elif args.pause_stop_exit_only:verify_pause_stop_exit(directory,inputs,report)
        else:verify(directory,inputs,report,args.total_budget_only)
        if any(read_stable(ROOT/name,1024*1024)!=data for name,data in inputs.items()):raise ValueError('canonical C3 changed during verification')
        report['current_sources_unchanged']=True;report['status']='passed';status=0
    except (OSError,ValueError,subprocess.SubprocessError) as error:
        report['status']='failed';report['error']=str(error)
    retain(directory/'run.json',canonical(report))
    print(json.dumps({'status':report['status'],'evidence':str(directory/'run.json'),'real_candidate_created':0,'runtime_approved':False},sort_keys=True))
    return status


if __name__=='__main__':
    sys.exit(main())
