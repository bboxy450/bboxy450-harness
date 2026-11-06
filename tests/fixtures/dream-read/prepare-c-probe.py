# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Compile the signed canonical probe; retain actual inputs, never run a candidate."""
import argparse
import datetime
import hashlib
import json
import os
from pathlib import Path
import platform
import shlex
import stat
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
SOURCE = 'tests/fixtures/dream-read/proof-helper-probe.c'
SCHEMA = 'tests/fixtures/dream-proof-tools/closure-boundary.schema.json'
VECTORS = 'tests/fixtures/dream-proof-tools/closure-boundary-vectors.json'
READSET = ['src/dream/' + n + '.rs' for n in ('resolve', 'trust', 'c1', 'read')]
PREPARATION = ['tests/fixtures/dream-read/prepare-c-probe.py',
               'tests/fixtures/dream-read/check-c-preparation.py', 'Makefile']
FIELDS = ('st_dev', 'st_ino', 'st_size', 'st_mtime_ns', 'st_ctime_ns', 'st_uid', 'st_mode', 'st_nlink')


def digest(wire):
    return hashlib.sha256(wire).hexdigest()


def run(argv):
    result = subprocess.run(argv, cwd=ROOT, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                            timeout=60, check=False)
    if result.returncode:
        raise RuntimeError(f'command {argv!r} exit {result.returncode}: {result.stderr.decode(errors="replace")}')
    return result.stdout


def stamp(info):
    return {f.removeprefix('st_'): getattr(info, f) for f in FIELDS}


def file_identity(path):
    """Bind a regular file without following a final symlink, checking its open FD."""
    fd = os.open(path, os.O_RDONLY | os.O_CLOEXEC | os.O_NOFOLLOW)
    try:
        before = os.fstat(fd)
        if not stat.S_ISREG(before.st_mode):
            raise ValueError('nonregular file: ' + str(path))
        with os.fdopen(fd, 'rb', closefd=False) as source:
            wire = source.read()
        after = os.fstat(fd)
        if stamp(before) != stamp(after) or stamp(path.lstat()) != stamp(after):
            raise ValueError('file changed while binding: ' + str(path))
        return dict(path=str(path), len=len(wire), sha256=digest(wire), **stamp(after))
    finally:
        os.close(fd)


def lexical_file_identity(path):
    """Keep the compiler's lexical include name and every traversed symlink."""
    lexical = Path(path)
    if not lexical.is_absolute():
        lexical = Path.cwd() / lexical
    pending = list(lexical.parts[1:])
    current = Path(lexical.anchor)
    links = []
    while pending:
        part = pending.pop(0)
        if part == '..':
            current = current.parent
            continue
        current = current / part
        before = current.lstat()
        if stat.S_ISLNK(before.st_mode):
            if len(links) >= 40:
                raise ValueError('symlink traversal bound: ' + str(lexical))
            target = os.readlink(current)
            wire = os.fsencode(target)
            if stamp(current.lstat()) != stamp(before):
                raise ValueError('symlink moved while binding: ' + str(current))
            links.append(dict(path=str(current), target=target, len=len(wire), sha256=digest(wire), **stamp(before)))
            replacement = Path(target)
            if replacement.is_absolute():
                current = Path(replacement.anchor)
                pending = list(replacement.parts[1:]) + pending
            else:
                current = current.parent
                pending = list(replacement.parts) + pending
        elif pending and not stat.S_ISDIR(before.st_mode):
            raise ValueError('non-directory include component: ' + str(current))
    identity = file_identity(current)
    identity['resolved_path'] = identity['path']
    identity['path'] = str(lexical)
    identity['symlinks'] = links
    return identity


def private_directory(path):
    """Only the dedicated preparation tree is repaired; its shared parent is untouched."""
    path.mkdir(mode=0o700, parents=False, exist_ok=True)
    before = path.lstat()
    if not stat.S_ISDIR(before.st_mode) or before.st_uid != os.geteuid():
        raise ValueError('foreign preparation directory: ' + str(path))
    if stat.S_IMODE(before.st_mode) != 0o700:
        path.chmod(0o700, follow_symlinks=False)
    after = path.lstat()
    if (before.st_dev, before.st_ino) != (after.st_dev, after.st_ino) or stat.S_IMODE(after.st_mode) != 0o700:
        raise ValueError('preparation directory changed: ' + str(path))
    return path


def retain(path, wire, mode=0o400):
    fd = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL | os.O_CLOEXEC | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'wb') as output:
        output.write(wire)
        output.flush()
        os.fsync(output.fileno())
        os.fchmod(output.fileno(), mode)
    return file_identity(path)


def snapshot_files(directory, wires):
    for name, wire in wires.items():
        path = directory / name
        relative = path.parent.relative_to(directory)
        parent = directory
        for part in relative.parts:
            parent = private_directory(parent / part)
        retain(path, wire)


def retained_compile_directory(target):
    return Path(tempfile.mkdtemp(prefix='compile-', dir=private_directory(target)))


def header_closure(dependencies, source):
    # Do not resolve(): SDK pthread.h is a lexical symlink whose identity is an
    # actual compile input, distinct from its pthread/pthread.h destination.
    names = shlex.split(dependencies.split(':', 1)[1].replace('\\\n', ''))
    paths = [Path(name) if Path(name).is_absolute() else Path.cwd() / name for name in names]
    source = Path(source) if Path(source).is_absolute() else Path.cwd() / source
    return {str(path): lexical_file_identity(path) for path in paths if path != source}


def retain_headers(headers, root):
    for name, identity in headers.items():
        if lexical_file_identity(Path(name)) != identity:
            raise ValueError('header moved before snapshot: ' + name)
        wire = Path(identity['resolved_path']).read_bytes()
        if len(wire) != identity['len'] or digest(wire) != identity['sha256']:
            raise ValueError('header bytes moved before snapshot: ' + name)
        copy = root / identity['sha256']
        if not copy.exists():
            retain(copy, wire)
        elif file_identity(copy)['sha256'] != identity['sha256']:
            raise ValueError('header snapshot collision')
        identity['snapshot'] = str(copy)
        for link in identity['symlinks']:
            copy = root / link['sha256']
            wire = os.fsencode(link['target'])
            if not copy.exists():
                retain(copy, wire)
            elif file_identity(copy)['sha256'] != link['sha256']:
                raise ValueError('symlink snapshot collision')
            link['snapshot'] = str(copy)


def validate_outputs(root):
    """Check every retained output, including the ACTUAL compiler argv paths."""
    outputs = []
    for path in [root, *sorted(root.rglob('*'))]:
        info = path.lstat()
        if info.st_uid != os.geteuid() or stat.S_ISLNK(info.st_mode):
            raise ValueError('output custody differs: ' + str(path))
        if stat.S_ISDIR(info.st_mode):
            if stat.S_IMODE(info.st_mode) != 0o700:
                raise ValueError('output directory is not0700: ' + str(path))
            outputs.append({'path': str(path), 'kind': 'directory', 'dev': info.st_dev,
                            'ino': info.st_ino, 'uid': info.st_uid, 'mode': info.st_mode})
        elif stat.S_ISREG(info.st_mode):
            if info.st_nlink != 1 or stat.S_IMODE(info.st_mode) not in (0o400, 0o500):
                raise ValueError('output file is not immutable private single-link: ' + str(path))
            outputs.append({'kind': 'file', **file_identity(path)})
        else:
            raise ValueError('unexpected output kind: ' + str(path))
    return outputs


def signed_commit(revision):
    commit = run(['git', 'rev-parse', '--verify', revision + '^{commit}']).decode().strip()
    run(['git', 'verify-commit', '--raw', commit])
    signer = run(['git', 'log', '-1', '--format=%GF', commit]).decode().strip()
    if signer != '9B07E849480C68328CA20684844ECC0587DE7AB0':
        raise ValueError('unexpected source signer')
    return commit, signer


def select_inputs(source_commit, preparation_commit, read_blob):
    source = {name: read_blob(source_commit, name) for name in (SOURCE, SCHEMA, VECTORS, *READSET)}
    preparation = {name: read_blob(preparation_commit, name) for name in PREPARATION}
    return source, preparation


def source_digest(wires):
    result = hashlib.sha256(b'bboxy.dream.proof-tools/source/1\0')
    for name in READSET:
        wire = wires[name]
        result.update(Path(name).name.encode() + b'\0' + len(wire).to_bytes(8, 'little') + wire)
    return result.hexdigest()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', required=True)
    parser.add_argument('--source-commit', default='HEAD', help='explicit signed frozen source/read-set commit')
    args = parser.parse_args()
    requested = Path(args.target_dir)
    if not requested.is_absolute():
        parser.error('--target-dir must be absolute')
    os.umask(0o077)
    head, signer = signed_commit('HEAD')
    frozen, source_signer = signed_commit(args.source_commit)
    source_wires, preparation_wires = select_inputs(frozen, head, lambda commit, name: run(['git', 'show', commit + ':' + name]))
    for name, wire in preparation_wires.items():
        if (ROOT / name).read_bytes() != wire:
            raise ValueError('preparation requires exact signed HEAD bytes: ' + name)
    if digest(source_wires[SCHEMA]) != 'd70948ec375dae099ea1dac0b92dfb30ff87d7fcac94c98f42caa1c0ba2da8ec':
        raise ValueError('foreign schema authority')
    if digest(source_wires[VECTORS]) != 'bff32c1afb483980952a6761918d4005c9f1e12b66f3d7aaaa88f1b9afe09740':
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
    requested.mkdir(parents=True, exist_ok=True)
    target = private_directory(requested.resolve() / 'dream-read-c-preparation')
    # This is intentionally NOT TemporaryDirectory: the exact paths consumed
    # and emitted by the actual compiler remain available to the builder.
    own = retained_compile_directory(target)
    source = own / 'proof-helper-probe.c'
    compile_input = retain(source, source_wires[SOURCE])
    dependency_argv = [str(compiler), *flags, '-M', '-MT', 'probe', str(source)]
    dependency_wire = run(dependency_argv)
    dependencies = dependency_wire.decode()
    dependency_output = retain(own / 'dependencies.log', dependency_wire)
    headers = header_closure(dependencies, source)
    argv = [str(compiler), '-v', *flags, str(source), '-o', str(own / 'probe')]
    build = subprocess.run(argv, cwd=own, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False)
    stdout_output = retain(own / 'build.stdout.log', build.stdout)
    stderr_output = retain(own / 'build.stderr.log', build.stderr)
    if build.returncode:
        raise RuntimeError(f'compile exit {build.returncode}: retained {stderr_output["path"]}')
    for path, identity in headers.items():
        if lexical_file_identity(Path(path)) != identity:
            raise ValueError('header inputs moved during actual compilation: ' + path)
    if file_identity(source) != compile_input:
        raise ValueError('actual compile input moved')
    executable = own / 'probe'
    executable.chmod(0o500)
    executable_identity = file_identity(executable)
    source_snapshot = private_directory(own / ('source-' + frozen))
    preparation_snapshot = private_directory(own / ('preparation-' + head))
    snapshot_files(source_snapshot, source_wires)
    snapshot_files(preparation_snapshot, preparation_wires)
    header_root = private_directory(own / 'compiler-headers')
    retain_headers(headers, header_root)
    for name, identity in headers.items():
        expected = {key: value for key, value in identity.items() if key != 'snapshot'}
        expected['symlinks'] = [{key: value for key, value in link.items() if key != 'snapshot'}
                                for link in identity['symlinks']]
        if lexical_file_identity(Path(name)) != expected:
            raise ValueError('header changed after snapshot: ' + name)
    if file_identity(compiler) != compiler_identity or file_identity(linker) != linker_identity:
        raise ValueError('compiler moved during preparation')
    for name, wire in preparation_wires.items():
        if (ROOT / name).read_bytes() != wire:
            raise ValueError('preparation source moved during compilation: ' + name)
    inputs = {name: {'len': len(wire), 'sha256': digest(wire)} for name, wire in {**source_wires, **preparation_wires}.items()}
    metadata = {
        'format': 'bboxy.dream.c-preparation/1',
        'source_commit': frozen, 'signature_verified': True, 'signer_fingerprint': source_signer,
        'preparation_commit': head, 'preparation_signature_verified': True, 'preparation_signer_fingerprint': signer,
        'created': datetime.datetime.now(datetime.timezone.utc).isoformat(),
        'schema_authority_commit': 'b26d7cd3432e9cae3e6c68492afd73a82cc8f2e9',
        'inputs': inputs, 'input_commits': {name: frozen if name in source_wires else head for name in inputs},
        'source_snapshot': str(source_snapshot), 'preparation_snapshot': str(preparation_snapshot),
        'retained_root': str(own), 'retained_directory_mode': '0700',
        'compiler': {**compiler_identity, 'version': version, 'declared_dylibs': compiler_dependencies,
                     'reported_target': triple, 'target': 'aarch64-apple-darwin', 'flags': ['-v', *flags]},
        'linker': linker_identity, 'header_inputs': headers,
        'dependency_discovery': {'argv': dependency_argv, 'exit_code': 0, 'stdout_sha256': digest(dependency_wire),
                                 'stdout': dependencies, 'retained_stdout': dependency_output},
        'sdk': {'path': str(sdk), 'settings': file_identity(sdk / 'SDKSettings.json'),
                'version': run(['/usr/bin/xcrun', '--sdk', 'macosx', '--show-sdk-version']).decode().strip()},
        'platform': {'machine': platform.machine(), 'release': platform.release(),
                     'os_build': run(['/usr/sbin/sysctl', '-n', 'kern.osversion']).decode().strip()},
        'build': {'argv': argv, 'cwd': str(own), 'exit_code': build.returncode, 'input': compile_input,
                  'output': executable_identity, 'stdout_sha256': digest(build.stdout), 'stderr_sha256': digest(build.stderr),
                  'stdout': build.stdout.decode(errors='strict'), 'stderr': build.stderr.decode(errors='strict'),
                  'retained_stdout': stdout_output, 'retained_stderr': stderr_output},
        'executable': executable_identity,
        'provisioning_source': {'domain': 'bboxy.dream.proof-tools/source/1\\0',
                                'sha256': source_digest(source_wires), 'kind': 'exact_source_snapshot',
                                'compiled_product_currentness_proven': False},
        'runtime_approved': False, 'basis': 'conditional_source_review',
        'whole_command_lifetime_proof': False, 'private_test': 'not_applicable_private_test', 'candidate_run': False,
        'limitations': ['Preparation compiles the probe only; it does not qualify Git/GPG, a Basis, a source premise or native lifetime.',
                        'The four Rust inputs are exact signed frozen bytes and the existing domain digest; no current product executable is built here.',
                        'Lexical headers, symlink target bytes, compiler/linker identities and actual retained invocation are recorded; this is not a transitive native/compiler lifetime theorem.']}
    metadata['retained_outputs'] = validate_outputs(own)
    wire = json.dumps(metadata, sort_keys=True, separators=(',', ':')).encode() + b'\n'
    record = own / ('preparation-' + digest(wire) + '.json')
    retain(record, wire)
    validate_outputs(own)
    private_directory(target)
    print(json.dumps({'record': str(record), 'sha256': digest(wire), 'source_commit': frozen,
                      'preparation_commit': head, 'probe_sha256': executable_identity['sha256'], 'runtime_approved': False}, sort_keys=True))


if __name__ == '__main__':
    main()
