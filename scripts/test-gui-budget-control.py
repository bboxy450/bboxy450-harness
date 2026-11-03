# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Inspired by bboxy450-harness production mutation controls in scripts/test-capability-control.py.
# https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""The real four-view witness must reject removal of its shared deadline guard."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/frontends/gui/budget.rs'
TEST = 'four_views_share_one_clock_drain_and_projection_without_losing_the_tail'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone,test-fixtures', '--test', 'gui_budget', TEST, '--', '--exact', '--nocapture']
GUARD = '        if spent >= budget {\n            break;\n        }\n'
original = SOURCE.read_bytes()
text = original.decode()
if text.count(GUARD) != 1:
    raise SystemExit('aggregate budget control: expected one production tail guard')
env = os.environ.copy()
try:
    SOURCE.write_text(text.replace(GUARD, '', 1))
    red = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
    assertion = 'aggregate projection exceeded 600us'
    if red.returncode != 101 or assertion not in red.stdout + red.stderr:
        raise RuntimeError('aggregate budget control did not fail its named assertion: ' + (red.stdout + red.stderr)[-3000:])
finally:
    SOURCE.write_bytes(original)
green = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
if green.returncode != 0:
    raise RuntimeError('restored aggregate budget witness failed: ' + (green.stdout + green.stderr)[-3000:])
if SOURCE.read_bytes() != original:
    raise RuntimeError('aggregate budget control did not restore the exact source')
print(json.dumps({'command': COMMAND, 'source': 'src/frontends/gui/budget.rs', 'assertion': assertion, 'red_exit': red.returncode, 'green_exit': green.returncode, 'restored_source_sha256': hashlib.sha256(original).hexdigest(), 'operator_input': False}))
