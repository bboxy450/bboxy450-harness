# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Adapted from scripts/test-gui-budget-control.py in this project.
"""Drop explicit launcher approval intent: the real CLI-to-ACP request must reject."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/frontends/headless.rs'
TEST = 'outer_cli_maps_yolo_and_invalid_config_to_downstream'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--test', 'acp_launch_context', TEST, '--', '--exact', '--nocapture']
GUARD = 'launch.yolo = yolo;'
OLD = 'launch.yolo = false;'
original = SOURCE.read_bytes()
text = original.decode()
if text.count(GUARD) != 1:
    raise SystemExit('launch approval control: expected one launcher approval assignment')
env = os.environ.copy()
logs = Path(tempfile.mkdtemp(prefix='harness-approval-control-', dir='/tmp'))
try:
    SOURCE.write_text(text.replace(GUARD, OLD, 1))
    red = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
    (logs / 'red.log').write_text(red.stdout + red.stderr)
    assertion = 'launcher --yolo approval semantics dropped'
    if red.returncode != 101 or assertion not in red.stdout + red.stderr:
        raise RuntimeError('launch approval control did not fail its named assertion: ' + (red.stdout + red.stderr)[-3000:])
finally:
    SOURCE.write_bytes(original)
green = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
(logs / 'green.log').write_text(green.stdout + green.stderr)
if green.returncode != 0:
    raise RuntimeError('restored launch approval witness failed: ' + (green.stdout + green.stderr)[-3000:])
if SOURCE.read_bytes() != original:
    raise RuntimeError('launch approval control did not restore exact source')
print(json.dumps({'command': COMMAND, 'source': 'src/frontends/headless.rs', 'assertion': assertion,
    'red_exit': red.returncode, 'green_exit': green.returncode,
    'restored_source_sha256': hashlib.sha256(original).hexdigest(),
    'logs': str(logs), 'operator_input': False}))
