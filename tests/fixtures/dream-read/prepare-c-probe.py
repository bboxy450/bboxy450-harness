# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Compile the signed canonical probe; freeze inputs, never run a candidate."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shlex
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SOURCE = 'tests/fixtures/dream-read/proof-helper-probe.c'
SCHEMA = 'tests/fixtures/dream-proof-tools/closure-boundary.schema.json'
VECTORS = 'tests/fixtures/dream-proof-tools/closure-boundary-vectors.json'
READSET = ['src/dream/' + n + '.rs' for n in ('resolve', 'trust', 'c1', 'read')]


def digest(wire):
    return hashlib.sha256(wire).hexdigest()


def run(argv):
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=60, check=False)
    if result.returncode:
        raise RuntimeError(f'command {argv!r} exit {result.returncode}: {result.stderr.decode(errors="replace")}')
    return result.stdout


def file_identity(path):
    before = path.stat()
    wire = path.read_bytes()
    after = path.stat()
    fields = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns', 'st_uid', 'st_mode', 'st_nlink')
    if any(getattr(before, f) != getattr(after, f) for f in fields):
        raise ValueError('file changed while binding: ' + str(path))
    return dict(path=str(path), len=len(wire), sha256=digest(wire),
                **{f.removeprefix('st_'): getattr(after, f) for f in fields})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', required=True)
    args = parser.parse_args()
    head = run(['git', 'rev-parse', 'HEAD']).decode().strip()
    run(['git', 'verify-commit', '--raw', head])
    signer = run(['git', 'log', '-1', '--format=%GF', head]).decode().strip()
    if signer != '9B07E849480C68328CA20684844ECC0587DE7AB0':
        raise ValueError('unexpected source signer')
    paths = [SOURCE, SCHEMA, VECTORS, *READSET, 'tests/fixtures/dream-read/prepare-c-probe.py', 'Makefile']
    inputs = {}
    wires = {}
    for name in paths:
        wire = (ROOT / name).read_bytes()
        if run(['git', 'show', head + ':' + name]) != wire:
            raise ValueError('preparation requires exact signed HEAD bytes: ' + name)
        wires[name] = wire
        inputs[name] = {'len': len(wire), 'sha256': digest(wire)}
    if inputs[SCHEMA]['sha256'] != 'd70948ec375dae099ea1dac0b92dfb30ff87d7fcac94c98f42caa1c0ba2da8ec':
        raise ValueError('foreign schema authority')
    if inputs[VECTORS]['sha256'] != 'bff32c1afb483980952a6761918d4005c9f1e12b66f3d7aaaa88f1b9afe09740':
        raise ValueError('foreign vector authority')
    compiler = Path(run(['/usr/bin/xcrun', '--find', 'clang']).decode().strip()).resolve()
    sdk = Path(run(['/usr/bin/xcrun', '--sdk', 'macosx', '--show-sdk-path']).decode().strip()).resolve()
    compiler_identity = file_identity(compiler)
    linker = Path(run(['/usr/bin/xcrun', '--find', 'ld']).decode().strip()).resolve()
    linker_identity = file_identity(linker)
    compiler_dependencies = run(['/usr/bin/otool', '-L', str(compiler)]).decode()
    version = run([str(compiler), '--version']).decode()
    triple = run([str(compiler), '-dumpmachine']).decode().strip()
    flags = ['-O2', '-std=c11', '-Wall', '-Wextra', '-Werror', '-target', 'arm64-apple-darwin', '-isysroot', str(sdk)]
    target = Path(args.target_dir).resolve() / 'dream-read-c-preparation'
    target.mkdir(parents=True, exist_ok=True)
    with tempfile.TemporaryDirectory(prefix='compile-', dir=target) as tmp:
        own = Path(tmp)
        source = own / 'proof-helper-probe.c'
        source.write_bytes(wires[SOURCE])
        dependencies = run([str(compiler), *flags, '-M', '-MT', 'probe', str(source)]).decode()
        header_paths = [Path(n).resolve() for n in shlex.split(dependencies.split(':', 1)[1].replace('\\\n', ''))]
        headers = {str(path): file_identity(path) for path in header_paths if path != source}
        argv = [str(compiler), '-v', *flags, str(source), '-o', str(own / 'probe')]
        build = subprocess.run(argv, cwd=own, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                               timeout=60, check=False)
        if build.returncode:
            raise RuntimeError(f'compile exit {build.returncode}: {build.stderr.decode(errors="replace")}')
        if any(file_identity(Path(path)) != identity for path, identity in headers.items()):
            raise ValueError('header inputs moved during actual compilation')
        exe = (own / 'probe').read_bytes()
        exe_sha = digest(exe)
        destination = target / ('probe-' + exe_sha)
        if destination.exists():
            if destination.read_bytes() != exe:
                raise ValueError('immutable executable collision')
        else:
            (own / 'probe').chmod(0o500)
            os.link(own / 'probe', destination)
    for name, wire in wires.items():
        if (ROOT / name).read_bytes() != wire:
            raise ValueError('source moved during preparation: ' + name)
    if file_identity(compiler) != compiler_identity or file_identity(linker) != linker_identity:
        raise ValueError('compiler moved during preparation')
    source_hash = hashlib.sha256(b'bboxy.dream.proof-tools/source/1\0')
    for name in READSET:
        wire = wires[name]
        source_hash.update(Path(name).name.encode() + b'\0' + len(wire).to_bytes(8, 'little') + wire)
    snapshot = target / ('source-' + head)
    snapshot.mkdir(mode=0o700, exist_ok=True)
    for name, wire in wires.items():
        path = snapshot / name
        path.parent.mkdir(parents=True, exist_ok=True)
        if path.exists():
            if path.read_bytes() != wire:
                raise ValueError('immutable source collision: ' + name)
        else:
            path.write_bytes(wire)
            path.chmod(0o400)
    header_root = snapshot / 'compiler-headers'
    header_root.mkdir(mode=0o700, exist_ok=True)
    for name, identity in headers.items():
        wire = Path(name).read_bytes()
        if digest(wire) != identity['sha256']:
            raise ValueError('header moved before snapshot: ' + name)
        copy = header_root / identity['sha256']
        if copy.exists():
            if copy.read_bytes() != wire:
                raise ValueError('header snapshot collision')
        else:
            copy.write_bytes(wire)
            copy.chmod(0o400)
        identity['snapshot'] = str(copy)
    metadata = {
        'format': 'bboxy.dream.c-preparation/1',
        'source_commit': head, 'signature_verified': True, 'signer_fingerprint': signer,
        'created': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'schema_authority_commit': 'b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9',
        'inputs': inputs, 'source_snapshot': str(snapshot),
        'compiler': {**compiler_identity, 'version': version, 'declared_dylibs': compiler_dependencies,
                     'reported_target': triple, 'target': 'aarch64-apple-darwin', 'flags': ['-v', *flags]},
        'linker': linker_identity, 'header_inputs': headers,
        'dependency_discovery': {'argv': [str(compiler), *flags, '-M', '-MT', 'probe', str(source)],
                                 'exit_code': 0, 'stdout_sha256': digest(dependencies.encode()), 'stdout': dependencies},
        'sdk': {'path': str(sdk), 'settings': file_identity(sdk / 'SDKSettings.json'),
                'version': run(['/usr/bin/xcrun', '--sdk', 'macosx', '--show-sdk-version']).decode().strip()},
        'platform': {'machine': platform.machine(), 'release': platform.release(),
                     'os_build': run(['/usr/sbin/sysctl', '-n', 'kern.osversion']).decode().strip()},
        'build': {'argv': argv, 'exit_code': build.returncode,
                  'stdout_sha256': digest(build.stdout), 'stderr_sha256': digest(build.stderr),
                  'stdout': build.stdout.decode(errors='strict'), 'stderr': build.stderr.decode(errors='strict')},
        'executable': file_identity(destination),
        'provisioning_source': {'domain': 'bboxy.dream.proof-tools/source/1\\0',
                                'sha256': source_hash.hexdigest(), 'kind': 'exact_source_snapshot',
                                'compiled_product_currentness_proven': False},
        'runtime_approved': False, 'basis': 'conditional_source_review',
        'whole_command_lifetime_proof': False, 'private_test': 'not_applicable_private_test',
        'candidate_run': False,
        'limitations': ['Preparation compiles the probe only; it does not qualify Git/GPG, a Basis, a source premise or native lifetime.',
                        'The four Rust inputs are exact signed bytes and the existing domain digest; no product executable is built here.',
                        'Compiler declared dylibs, exact compiler/linker files and actual verbose invocation are recorded; this is not a transitive native/compiler lifetime theorem.']}
    wire = json.dumps(metadata, sort_keys=True, separators=(',', ':')).encode() + b'\n'
    record = target / ('preparation-' + digest(wire) + '.json')
    with record.open('xb') as output:
        output.write(wire)
    record.chmod(0o400)
    print(json.dumps({'record': str(record), 'sha256': digest(wire), 'source_commit': head,
                      'probe_sha256': exe_sha, 'runtime_approved': False}, sort_keys=True))


if __name__ == '__main__':
    main()
