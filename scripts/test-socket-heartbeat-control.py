# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Adapted from scripts/test-gui-budget-control.py in this project.
"""Restore the old routing filter: a real silent durable-only socket must fail."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/gateway/mod.rs'
TEST = 'idle_durable_subscriber_keeps_transport_fresh_without_semantic_activity'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'test-fixtures', '--test', 'gateway_unix', TEST, '--', '--exact', '--nocapture']
GUARD = 'if e.seq.is_some() || sub.ephemeral || matches!(e.event, Event::Status { .. }) {'
OLD = 'if e.seq.is_some() || sub.ephemeral {'
original = SOURCE.read_bytes()
text = original.decode()
if text.count(GUARD) != 1:
    raise SystemExit('heartbeat control: expected one Status routing guard')
env = os.environ.copy()
logs = Path(tempfile.mkdtemp(prefix='harness-heartbeat-control-', dir='/tmp'))
try:
    SOURCE.write_text(text.replace(GUARD, OLD, 1))
    red = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
    (logs / 'red.log').write_text(red.stdout + red.stderr)
    assertion = 'durable-only subscriber filtered the idle heartbeat'
    if red.returncode != 101 or assertion not in red.stdout + red.stderr:
        raise RuntimeError('heartbeat control did not fail its named assertion: ' + (red.stdout + red.stderr)[-3000:])
    if env.get('BBOXY_TERM_HEARTBEAT_PROBE') and 'actual term consumer filtered the15s idle heartbeat before its20s expiry' not in red.stdout + red.stderr:
        raise RuntimeError('actual term consumer did not reject the missing heartbeat before reconnect could mask it')
finally:
    SOURCE.write_bytes(original)
green = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=180)
(logs / 'green.log').write_text(green.stdout + green.stderr)
if green.returncode != 0:
    raise RuntimeError('restored idle heartbeat witness failed: ' + (green.stdout + green.stderr)[-3000:])
if SOURCE.read_bytes() != original:
    raise RuntimeError('heartbeat control did not restore exact source')
print(json.dumps({'command': COMMAND, 'source': 'src/gateway/mod.rs', 'assertion': assertion,
    'red_exit': red.returncode, 'green_exit': green.returncode,
    'restored_source_sha256': hashlib.sha256(original).hexdigest(),
    'actual_term_consumer': bool(env.get('BBOXY_TERM_HEARTBEAT_PROBE')), 'logs': str(logs), 'operator_input': False}))
