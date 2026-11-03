# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Restore the old character-only dispatch, require regressions, then restore and pass."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/frontends/gui/window.rs'
OUT = ROOT / 'target/space-fix/controls'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone',
           '--test', 'gui_rich', 'named_space_key_', '--', '--nocapture']
REGRESSIONS = ['named_space_key_preserves_' + field for field in
               ['the_composer_draft', 'the_profile_search', 'slash_profile_input']]

def run(env, name):
    result = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True,
                            text=True, timeout=600)
    output = result.stdout + result.stderr
    (OUT / (name + '.log')).write_text(output)
    return result.returncode, output

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    source = SOURCE.read_text()
    digest = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
    match = 'Key::Character(_) | Key::Named(NamedKey::Space)'
    if source.count(match) != 2:
        raise ValueError('expected exactly two named-Space text-entry arms')
    env = os.environ.copy()
    original_home = Path(env['HOME'])
    env['CARGO_HOME'] = env.get('CARGO_HOME', str(original_home / '.cargo'))
    env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original_home / '.rustup'))
    env.pop('BBOXY_CAPABILITY', None)
    env.pop('BBOXY_CAPABILITY_FILE', None)
    with tempfile.TemporaryDirectory(prefix='harness-space-control-', dir='/tmp') as own:
        env['HOME'] = own
        env['TMPDIR'] = own
        try:
            SOURCE.write_text(source.replace(match, 'Key::Character(_)'))
            red, output = run(env, 'old-match')
        finally:
            SOURCE.write_text(source)
        assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == digest
        assert red == 101, f'old match: expected exit 101, got {red}'
        for name in REGRESSIONS:
            assert f'test {name} ... FAILED' in output, name
        green, _ = run(env, 'restored')
    record = {'command': COMMAND, 'source': str(SOURCE.relative_to(ROOT)),
              'restored_sha256': digest, 'old_match_exit': red,
              'restored_exit': green, 'required_failures': REGRESSIONS,
              'private_home': True, 'operator_input': False}
    (OUT / 'results.json').write_text(json.dumps(record, indent=2) + '\n')
    assert green == 0, f'restored match: expected exit 0, got {green}'
    print('named Space control: old match exit 101 (all three regressions); restored exit 0')

if __name__ == '__main__':
    main()
