# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Unattended GUI feature gate, isolated from user profiles and credentials."""
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
ROOT = Path(__file__).resolve().parents[1]
env = os.environ.copy()
original = Path(env['HOME'])
env['CARGO_HOME'] = env.get('CARGO_HOME', str(original / '.cargo'))
env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original / '.rustup'))
env['CLIPPY_CONF_DIR'] = str(ROOT / 'src/frontends/gui/lint')
env.pop('BBOXY_CAPABILITY', None)
env.pop('BBOXY_CAPABILITY_FILE', None)
rows = []
with tempfile.TemporaryDirectory(prefix='harness-gui-tests-', dir='/tmp') as own:
    env['HOME'] = own
    env['TMPDIR'] = own
    for command in [
        ['python3', '-I', 'tests/gui_dependencies.py'],
        ['cargo', 'clippy', '--offline', '--locked', '--all-features', '--lib', '--bin', 'bboxy450-harness', '--', '-D', 'warnings'],
        ['cargo', 'test', '--offline', '--locked', '--features', 'standalone,test-fixtures', '--test', 'gui_rich', '--test', 'gui_render', '--test', 'gui_bridge', '--test', 'gui_fixtures', '--test', 'gui_budget', '--test', 'gui_cli', '--test', 'capability_delivery', '--', '--nocapture'],
        ['make', 'test-gui-smoke'],
    ]:
        print('gui-test: ' + ' '.join(command), flush=True)
        result = subprocess.run(command, cwd=ROOT, env=env)
        rows.append({'command': command, 'exit_code': result.returncode})
        if result.returncode:
            break
record = {'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
          'private_home': True, 'operator_input': False, 'commands': rows}
(ROOT / '.bboxy/runs/gui-gate.json').write_text(json.dumps(record, indent=2) + '\n')
sys.exit(rows[-1]['exit_code'])
