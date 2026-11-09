# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Unattended source-known shim process controls; no builder/qualification run."""
import argparse
import errno
import hashlib
import json
import os
from pathlib import Path
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[3]
FIX = Path(__file__).resolve().parent
FLAGS = ['-O2', '-std=c11', '-Wall', '-Wextra', '-Werror']
CONF = b'batch\nno-tty\nno-autostart\nno-auto-key-retrieve\nno-auto-key-import\nauto-key-locate clear\nno-auto-check-trustdb\n'


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--target-dir', required=True)
    args = parser.parse_args()
    parent = Path(args.target_dir).resolve() / 'exec-shim-controls'
    parent.mkdir(parents=True, exist_ok=True)
    run = Path(tempfile.mkdtemp(prefix='run-', dir=parent))
    os.umask(0o077)
    commands = []

    def command(argv, **kw):
        p = subprocess.run([str(a) for a in argv], capture_output=True, timeout=20, **kw)
        commands.append(dict(argv=[str(a) for a in argv], exit_code=p.returncode,
                             stdout=p.stdout.decode(errors='replace'), stderr=p.stderr.decode(errors='replace')))
        assert p.returncode == 0, commands[-1]
        return p.stdout

    try:
        for source, name in [('dream-exec-shim.c', 'dream-exec-shim'),
                             ('exec-shim-controls.c', 'controls'), ('exec-shim-tool.c', 'tool')]:
            command(['/usr/bin/clang', *FLAGS, FIX / source, '-o', run / name])
        for name in ['burn-tool', 'fork-tool', 'noop-tool']:
            shutil.copyfile(run / 'tool', run / name)
            (run / name).chmod(0o500)
        escape_modes = ['cleanup-esrch', 'cleanup-eio', 'census-esrch', 'census-eio', 'escaped-live', 'alive-after-reap']
        for mode in escape_modes:
            ledger = run / (mode + '.ndjson')
            result = json.loads(command([run / 'controls', 'escape-control', mode, ledger]))
            positive = mode in ['escaped-live', 'alive-after-reap']
            assert result['refused'] is positive and result['settled'] is not positive, result
            records = [json.loads(line) for line in ledger.read_text().splitlines()]
            diagnostics = [r for r in records if r.get('kind') == 'escape_check_unavailable']
            if mode.endswith('eio'):
                site = 'escaped_cleanup_bsd' if mode.startswith('cleanup') else 'group_census'
                assert any(r['site'] == site and r['errno'] == errno.EIO for r in diagnostics), records
            else:
                assert not diagnostics, records
        # Reproduce INPUT70's failure through the real production wrapper.
        source = (FIX / 'proof-helper-probe.c').read_bytes()
        anchor = b'if(f->site==ENGINE_OBS_ESCAPED_CLEANUP_BSD&&(o->shim_active||o->direct_reaped))'
        assert source.count(anchor) == 1
        mutant = run / 'mutant'
        mutant.mkdir(mode=0o700)
        (mutant / 'proof-helper-probe.c').write_bytes(source.replace(anchor, anchor.replace(b'if(', b'if(0&&', 1)))
        shutil.copyfile(FIX / 'exec-shim-controls.c', mutant / 'exec-shim-controls.c')
        shutil.copyfile(FIX / 'dream-exec-shim.h', mutant / 'dream-exec-shim.h')
        command(['/usr/bin/clang', *FLAGS, mutant / 'exec-shim-controls.c', '-o', mutant / 'controls'])
        broken = subprocess.run([str(mutant / 'controls'), 'escape-control', 'cleanup-esrch', str(run / 'mutant.ndjson')], capture_output=True, timeout=20)
        commands.append(dict(argv=[str(mutant / 'controls'), 'escape-control', 'cleanup-esrch'], exit_code=broken.returncode,
                             stdout=broken.stdout.decode(errors='replace'), stderr=broken.stderr.decode(errors='replace'), expected_exit_code=81))
        assert broken.returncode == 81 and b'postreap escape read settles without terminal-proof route' in broken.stderr, commands[-1]
        home, scratch = run / 'home', run / 'scratch'
        home.mkdir(mode=0o700)
        scratch.mkdir(mode=0o700)
        (home / 'gpg.conf').write_bytes(CONF)
        shim = run / 'dream-exec-shim'
        signature = b'public source-known signature bytes\n'
        signature_hash = hashlib.sha256(signature).hexdigest()

        def invoke(name, form, tail, *, git='noop-tool', gpg='noop-tool', environment=None):
            argv = [run / 'controls', shim, run / git, run / gpg, home, scratch, signature_hash,
                    str(len(signature)), run, run / (name + '.out'), str(form), *tail]
            raw = command(argv, env={**os.environ, **(environment or {})})
            result = json.loads(raw)
            assert result['runtime_approved'] is False
            return result

        prefix = ['--no-pager', '--no-optional-locks', '-c', 'core.fsmonitor=false',
                  '-c', 'core.hooksPath=/dev/null', '-c', 'credential.helper=', '-c', 'protocol.allow=never',
                  '-c', 'gpg.format=openpgp', '-c', 'gpg.program=' + str(shim),
                  '-c', 'gpg.openpgp.program=' + str(shim)]
        result = invoke('allowed', 0, [*prefix, 'rev-parse', '--is-bare-repository'])
        assert result['status'] == 0 and result['receipts'] == 1, result
        result = invoke('failed-live-read', 0, [*prefix, 'rev-parse', '--is-bare-repository'],
                        environment={'DREAM_SHIM_TEST_FAILED_LIVE_READ': '1'})
        assert result['status'] == 0 and result['failed_live_reads_ignored'] == 1, result
        for name, variable in [('environment', 'DREAM_SHIM_TEST_BAD_ENV'), ('qos', 'DREAM_SHIM_TEST_BAD_QOS')]:
            result = invoke(name, 0, [*prefix, 'rev-parse', '--is-bare-repository'], environment={variable: '1'})
            assert result['status'] == 81 and result['receipts'] == 0, result
        result = invoke('argv-daemon' , 13, ['--daemon'])
        assert result['status'] == 81 and result['receipts'] == 0, result
        sig = scratch / '.git_vtag_tmpABC123'
        sig.write_bytes(signature)
        verify = ['--keyid-format=long', '--status-fd=1', '--verify', str(sig), '-']
        sig.chmod(0o644)
        result = invoke('bad-custody', 12, verify)
        assert result['status'] == 81 and result['receipts'] == 0, result
        sig.chmod(0o600)
        result = invoke('gs1-gs2', 12, verify, environment={'DREAM_SHIM_TEST_UNLINK': '1'})
        assert result['status'] == 0 and result['gs1'] == 1 and result['gs2'] == 1, result
        for name, live in [('cpu-terminal', False), ('cpu-live', True)]:
            result = invoke(name, 0, [*prefix, 'rev-parse', '--is-bare-repository'], git='burn-tool',
                            environment={'DREAM_SHIM_TEST_CPU_BACKSTOP': '1'} if live else {})
            assert result['terminal_refused'] and result['wait4_cpu_ns'] > 250_000_000, result
            assert result['live_kill'] == int(live), result
        result = invoke('observed-extra-fork', 12,
                        [*prefix, 'log', '-1', '--format=%G?%x00%GF%x00%GP', 'a' * 40], git='fork-tool',
                        environment={'DREAM_SHIM_TEST_FORK_BACKSTOP': '1'})
        assert result['status'] == -1 and result['observed_fork_notes'] == 2, result
        print(json.dumps(dict(result='pass', controls=17, escape_controls=6, escape_mutants=1, native_process=True, source_known_tools=True,
                              cryptographic_proof=False, runtime_approved=False, record=str(run / 'result.json'))))
    finally:
        (run / 'result.json').write_text(json.dumps(dict(commands=commands, runtime_approved=False,
                                                        builder_run=False), indent=2) + '\n')


if __name__ == '__main__':
    main()
