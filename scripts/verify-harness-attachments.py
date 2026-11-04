#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Named attachment witnesses and only their finite ownership model/reference parity."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get('TARGET_DIR', ROOT / 'target')).resolve()
OUT = TARGET / 'harness-attachments'
OUT.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, RUSTC_WRAPPER='', CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='2', ATTACHMENTS_FORMAL_OUT=str(OUT / 'formal'))
records = []


def run(name, args):
    command = ['cargo', *args, '--offline', '--locked', '--jobs', '2', '--target-dir', str(TARGET)] if args[0] == 'check' else args
    log = OUT / (name + '.log')
    with log.open('w') as out:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=out, stderr=subprocess.STDOUT)
    records.append(dict(command=command, exit_code=result.returncode, log=str(log), sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
    print(f'harness-attachments: {name}: exit {result.returncode}', flush=True)
    if result.returncode:
        print(log.read_text()[-7000:], file=sys.stderr)
        raise SystemExit(result.returncode)


cargo = ['--offline', '--locked', '--jobs', '2', '--target-dir', str(TARGET), '-p', 'bboxy450-harness']
run('composer-acp', ['cargo', 'nextest', 'run', *cargo, '--features', 'standalone', '--test', 'harness_attachments'])
run('existing-pending-contract', ['cargo', 'nextest', 'run', *cargo, '--features', 'embedded', '--lib', '-E', 'test(gui_polish_pending_requires_causal_durable_user_in_current_birth) | test(attachment_unknown_delivery_keeps_real_blocks_and_custody) | test(image_intake_proof_rejects_durable_replay_old_epoch_and_reconnect_challenge)'])
run('existing-reconnect', ['cargo', 'nextest', 'run', *cargo, '--features', 'embedded', '--test', 'gui_bridge', '-E', 'test(stale_unknown_dispatch_does_not_hold_detach_on_the_replacement_session)'])
run('ownership-model', ['cargo', 'test', *cargo, '--test', 'formal_models', 'attachments_intake_native_model_and_control', '--', '--exact'])
run('strict-producer', ['cargo', 'clippy', *cargo, '--features', 'standalone', '--lib', '--test', 'harness_attachments', '--test', 'formal_models', '--test', 'gui_bridge', '--no-deps', '--', '-D', 'warnings'])
run('embedded', ['check', '-p', 'bboxy450-harness', '--no-default-features', '--features', 'embedded', '--lib'])

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('formal', ROOT / 'scripts/formal.py')
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
java, jar, pins = formal.tools(TARGET / 'formal')
models = []
for line in (OUT / 'formal/manifest.tsv').read_text().splitlines():
    name, expected, count, width = line.split('\t')
    assert name in {'HarnessAttachmentIntake', 'BrokenHarnessAttachmentIntake', 'HarnessAttachmentCapability', 'BrokenHarnessAttachmentCapability'}
    graph = formal.tlc(java, jar, OUT / 'formal', name, True)
    assert formal.verdict(graph) == 'pass'
    actual = formal.dot_graph(OUT / 'formal' / (name + '.dot'), int(width))
    assert actual == formal.native_graph(OUT / 'formal', name) and len(actual[0]) == int(count)
    checked = formal.tlc(java, jar, OUT / 'formal', name, False)
    assert formal.verdict(checked) == expected
    models.append(dict(name=name, expected=expected, states=len(actual[0]), transitions=len(actual[1]), graph_equal=True, tlc_exit=checked.returncode))
    print(f'harness-attachments: {name}: {expected}; {len(actual[0])} states / {len(actual[1])} transitions match', flush=True)
assert len(models) == 4
(OUT / 'producer.json').write_text(json.dumps(dict(commands=records, models=models, pins=pins, operator_input=False, arbitrary_rust_refinement=False), indent=2) + '\n')
