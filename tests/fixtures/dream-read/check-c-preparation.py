# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Check private preparation custody, lexical headers and frozen source selection."""
import argparse
import json
import os
from pathlib import Path
import runpy
import stat
import subprocess
import sys
import tempfile

PREPARE = Path(__file__).with_name('prepare-c-probe.py')
M = runpy.run_path(str(PREPARE))


def require(ok, label):
    if not ok:
        raise ValueError(label)


def command(argv, cwd, label, report):
    result = subprocess.run(argv, cwd=cwd, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=60, check=False)
    report['commands'].append({'argv': argv, 'cwd': str(cwd), 'exit_code': result.returncode,
                               'stdout': M['retain'](cwd / (label + '.stdout.log'), result.stdout),
                               'stderr': M['retain'](cwd / (label + '.stderr.log'), result.stderr)})
    require(result.returncode == 0, label + ': actual nonzero exit')
    return result.stdout


def controls(root, report):
    case = M['private_directory'](root / 'custody')
    legacy = case / 'legacy0755'
    legacy.mkdir(mode=0o700)
    legacy.chmod(0o755)
    M['private_directory'](legacy)
    require(stat.S_IMODE(legacy.lstat().st_mode) == 0o700, 'legacy preparation root must become0700')
    link = case / 'planted-directory'
    link.symlink_to(legacy, target_is_directory=True)
    refused = False
    try:
        M['private_directory'](link)
    except ValueError:
        refused = True
    require(refused and link.is_symlink(), 'directory symlink must be refused without changing its target')
    link.unlink()
    report['assertions'].append('private0700 preparation root, symlink directory refused')

    case = M['private_directory'](root / 'lexical')
    canonical = M['private_directory'](case / 'pthread')
    wire = b'typedef void *pthread_t;\n'
    M['retain'](canonical / 'pthread.h', wire)
    alias = case / 'pthread.h'
    alias.symlink_to('pthread/pthread.h')
    forwarder = case / 'forwarder.h'
    M['retain'](forwarder, b'#include "pthread.h"\n')
    source = case / 'fixture.c'
    M['retain'](source, b'#include "forwarder.h"\n')
    dependencies = 'probe: ' + ' '.join(str(p) for p in (source, forwarder, alias, canonical / 'pthread.h')) + '\n'
    headers = M['header_closure'](dependencies, source)
    require(set(headers) == {str(forwarder), str(alias), str(canonical / 'pthread.h')}, 'lexical alias and forwarder must not be coalesced')
    identity = headers[str(alias)]
    require(identity['resolved_path'] == str(canonical / 'pthread.h') and identity['sha256'] == M['digest'](wire)
            and len(identity['symlinks']) == 1 and identity['symlinks'][0]['target'] == 'pthread/pthread.h',
            'lexical symlink target bytes and terminal header must be cross-bound')
    header_root = M['private_directory'](case / 'retained-headers')
    M['retain_headers'](headers, header_root)
    require(Path(identity['snapshot']).read_bytes() == wire
            and Path(identity['symlinks'][0]['snapshot']).read_bytes() == b'pthread/pthread.h', 'header and alias target snapshots must contain actual bytes')
    # Retain the negative path fact, then remove only this test's own symlink
    # before validating the immutable output tree.
    M['retain'](case / 'identities.json', json.dumps(headers, sort_keys=True).encode() + b'\n')
    alias.unlink()
    report['assertions'].append('lexical pthread.h alias, regular forwarder and terminal bytes all retained separately')

    frozen = {name: ('frozen:' + name).encode() for name in (M['SOURCE'], M['SHIM'], M['SHIM_HEADER'], M['SCHEMA'], M['VECTORS'], *M['READSET'])}
    current = {name: ('current:' + name).encode() for name in (*frozen, *M['PREPARATION'])}
    selected, preparation = M['select_inputs']('frozen', 'current', lambda commit, name: (frozen if commit == 'frozen' else current)[name])
    require(selected == frozen and all(preparation[name] == current[name] for name in M['PREPARATION']),
            'explicit frozen source/read-set must remain independent of current preparation source')
    require(M['source_digest'](selected) != M['source_digest'](current), 'moving current Rust bytes must not silently substitute for frozen source-domain bytes')
    report['assertions'].append('frozen source/read-set and current signed preparation commit remain distinct')

    # Actual tiny public C compilation only; never compile/run an unsigned
    # canonical probe here and never execute even this synthetic executable.
    case = M['retained_compile_directory'](root / 'retained-builds')
    compiler = Path(command(['/usr/bin/xcrun', '--find', 'clang'], case, 'find-clang', report).decode().strip()).resolve()
    sdk = Path(command(['/usr/bin/xcrun', '--sdk', 'macosx', '--show-sdk-path'], case, 'find-sdk', report).decode().strip()).resolve()
    source = case / 'fixture.c'
    source_identity = M['retain'](source, b'#include <pthread.h>\nint main(void){return 0;}\n')
    flags = ['-O2', '-std=c11', '-Wall', '-Wextra', '-Werror', '-target', 'arm64-apple-darwin', '-isysroot', str(sdk)]
    dependencies = command([str(compiler), *flags, '-M', '-MT', 'probe', str(source)], case, 'dependencies', report).decode()
    headers = M['header_closure'](dependencies, source)
    pthread = [v for k, v in headers.items() if Path(k).name == 'pthread.h' and Path(k).parent.name != 'pthread']
    require(len(pthread) == 1, 'actual SDK dependency must retain lexical pthread.h')
    if (sdk / 'usr/include/pthread.h').is_symlink():
        require(pthread[0]['symlinks'], 'actual SDK pthread alias identity must be retained')
    header_root = M['private_directory'](case / 'headers')
    M['retain_headers'](headers, header_root)
    output = case / 'probe'
    command([str(compiler), *flags, str(source), '-o', str(output)], case, 'compile', report)
    output.chmod(0o500)
    require(M['file_identity'](source) == source_identity and output.exists(), 'actual compiler input and output must persist after compilation')
    report['synthetic_compile'] = {'input': source_identity, 'output': M['file_identity'](output), 'executed': False,
                                   'pthread_header': pthread[0], 'header_count': len(headers)}
    M['retain'](case / 'headers.json', json.dumps(headers, sort_keys=True).encode() + b'\n')
    report['assertions'].append('actual tiny compile paths persist, actual SDK pthread lexical alias retained, executable never run')
    report['retained_outputs'] = M['validate_outputs'](root)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', required=True)
    args = parser.parse_args()
    target = Path(args.target_dir)
    if not target.is_absolute():
        parser.error('--target-dir must be absolute')
    os.umask(0o077)
    target.mkdir(parents=True, exist_ok=True)
    root = Path(tempfile.mkdtemp(prefix='c-preparation-controls-', dir=target)).resolve()
    report = {'format': 'bboxy.dream.c-preparation-controls/1', 'candidate_run': False,
              'signed_preparation': False, 'runtime_approved': False, 'commands': [], 'assertions': []}
    status = 1
    try:
        report['control_inputs'] = [M['file_identity'](p) for p in (PREPARE, Path(__file__))]
        controls(root, report)
        report['status'] = 'passed'
        status = 0
    except Exception as error:
        report['status'] = 'failed'
        report['error'] = str(error)
    record = root / 'run.json'
    M['retain'](record, json.dumps(report, sort_keys=True, separators=(',', ':')).encode() + b'\n')
    print(json.dumps({'status': report['status'], 'record': str(record), 'candidate_run': False}))
    return status


if __name__ == '__main__':
    sys.exit(main())
