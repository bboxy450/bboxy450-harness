# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Launcher argument/default-profile contracts, isolated from all user state."""
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
plist = plistlib.loads((ROOT / 'packaging/macos/Info.plist').read_bytes())
assert plist['CFBundleName'] == plist['CFBundleDisplayName'] == 'bboxy450-harness'
assert plist['CFBundleIdentifier'] == 'com.bboxy450.harness'
assert '.with_title("bboxy450-harness")' in (ROOT / 'src/frontends/gui/window.rs').read_text()
bundle = subprocess.run(['make', '-s', '-f', 'Makefile', '-f', '-', 'print-bundle'],
                        input='print-bundle:\n\t@echo "$(BUNDLE)"\n', cwd=ROOT,
                        capture_output=True, text=True, check=True, timeout=10).stdout.strip()
assert bundle == 'target/bboxy450-harness.app', bundle
with tempfile.TemporaryDirectory(prefix='harness launcher ') as private:
    root = Path(private)
    macos = root / 'bboxy450-harness.app/Contents/MacOS'
    macos.mkdir(parents=True)
    launcher = macos / 'launcher'
    shutil.copyfile(ROOT / 'packaging/macos/launcher.sh', launcher)
    launcher.chmod(0o755)
    binary = macos / 'bboxy450-harness'
    binary.write_text('#!/bin/sh\nprintf "%s\\n" "$@"\n')
    binary.chmod(0o755)
    env = {'HOME': private, 'PATH': '/usr/bin:/bin'}

    def run(args=(), profile=None):
        current = env.copy()
        if profile is not None:
            current['BBOXY450_HARNESS_PROFILE'] = profile
        result = subprocess.run([str(launcher), *args], env=current,
                                capture_output=True, text=True, check=True, timeout=5)
        return result.stdout.splitlines()

    assert run() == ['gui', '--cwd', private, '--profile', 'nemotron']
    defaults = root / '.bboxy450-harness/default-profile'
    defaults.parent.mkdir()
    defaults.write_text('  own-profile  \n')
    assert run()[-1] == 'own-profile'
    assert run(profile='env-profile')[-1] == 'env-profile'
    assert run(profile=' \t')[-1] == 'own-profile'
    defaults.write_text(' \n')
    assert run()[-1] == 'nemotron'
    args = ['gui', '--profile', 'explicit', '--cwd', '/project with spaces', '--help']
    assert run(args, 'ignored') == args
    assert run(['--export-schema']) == ['--export-schema']

    # Exercise the real signing recipe with private tool fixtures: unavailable
    # identity and signing failure must both stop, with no ad-hoc fallback.
    tools = root / 'tools'
    tools.mkdir()
    security = tools / 'security'
    codesign = tools / 'codesign'
    calls = root / 'codesign-calls'
    codesign.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$SIGN_CALLS"\nexit 17\n')
    security.write_text('#!/bin/sh\nexit 1\n')
    codesign.chmod(0o755)
    security.chmod(0o755)
    signing_env = {**env, 'PATH': str(tools) + ':/usr/bin:/bin', 'SIGN_CALLS': str(calls)}

    def sign():
        return subprocess.run(['make', '-s', '-o', 'app-files', 'app'], cwd=ROOT,
                              env=signing_env, capture_output=True, text=True, timeout=10)

    result = sign()
    assert result.returncode != 0 and 'identity unavailable' in result.stderr, result
    assert not calls.exists(), 'missing identity attempted signing'
    identity = '3943AF130A015B36824B7FBC86AA672E7B25F3C4'
    security.write_text(f'#!/bin/sh\nprintf "%s\\n" "{identity}"\n')
    result = sign()
    assert result.returncode != 0, result
    assert len(calls.read_text().splitlines()) == 1, 'failed signing continued'
    assert f'--sign {identity} ' in calls.read_text(), 'signing used another identity'
print('app-packaging: PASS; canonical app name, default precedence, spaced paths, CLI passthrough, signing refusals')
