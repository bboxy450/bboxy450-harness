#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Adapted from scripts/verify-harness-attachments.py in this project: focused named checks and graph comparison.
"""Private onboarding witnesses and only the new install lifecycle/reference model."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get('TARGET_DIR', ROOT / 'target')).resolve()
OUT = TARGET / 'harness-onboarding'
OUT.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, RUSTC_WRAPPER='', CARGO_PROFILE_DEV_DEBUG='0', CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='2', NEXTEST_TEST_THREADS='2', ONBOARDING_FORMAL_OUT=str(OUT / 'formal'), ONBOARDING_EVIDENCE_DIR=str(OUT / 'startup'), ONBOARDING_CAPTURE_DIR=str(OUT / 'captures'), CLIPPY_CONF_DIR=str(ROOT / 'src/frontends/gui/lint'))
commands = []


def run(name, command):
    log = OUT / (name + '.log')
    with log.open('w') as stream:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    commands.append(dict(name=name, command=command, exit_code=result.returncode, log=str(log), sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
    print(f'harness-onboarding: {name}: exit {result.returncode}', flush=True)
    if result.returncode:
        print(log.read_text()[-7000:], file=sys.stderr)
        raise SystemExit(result.returncode)


cargo = ['--offline', '--locked', '--jobs', '2', '--target-dir', str(TARGET), '-p', 'bboxy450-harness']
run('private-discovery-profile-installer-native-gui', ['cargo', 'nextest', 'run', *cargo, '--features', 'standalone', '--test', 'harness_onboarding'])
run('markdown-native-source-queued-history-and-causal-guard', ['cargo', 'nextest', 'run', *cargo, '--features', 'standalone', '--lib', '-E', 'test(onboarding_) | test(gui_polish_pending_requires_causal_durable_user_in_current_birth)'])
run('install-model-and-control', ['cargo', 'test', *cargo, '--test', 'formal_models', 'onboarding_install_native_model_and_control', '--', '--exact'])
run('strict-producer-and-standalone', ['cargo', 'clippy', *cargo, '--features', 'standalone', '--lib', '--bin', 'bboxy450-harness', '--test', 'harness_onboarding', '--test', 'formal_models', '--no-deps', '--', '-D', 'warnings'])
run('embedded-without-acp', ['cargo', 'check', *cargo, '--no-default-features', '--features', 'embedded', '--lib'])
run('license-headers', ['make', 'test-licenses'])
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('formal', ROOT / 'scripts/formal.py')
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
java, jar, pins = formal.tools(TARGET / 'formal')
models = []
for line in (OUT / 'formal/manifest.tsv').read_text().splitlines():
    name, expected, count, width = line.split('\t')
    assert name in {'HarnessOnboardingInstall', 'BrokenHarnessOnboardingInstall'}
    graph = formal.tlc(java, jar, OUT / 'formal', name, True)
    assert formal.verdict(graph) == 'pass'
    actual = formal.dot_graph(OUT / 'formal' / (name + '.dot'), int(width))
    assert actual == formal.native_graph(OUT / 'formal', name) and len(actual[0]) == int(count)
    checked = formal.tlc(java, jar, OUT / 'formal', name, False)
    assert formal.verdict(checked) == expected
    models.append(dict(name=name, expected=expected, states=len(actual[0]), transitions=len(actual[1]), graph_equal=True, tlc_exit=checked.returncode))
    print(f'harness-onboarding: {name}: {expected}; {len(actual[0])} states / {len(actual[1])} transitions match', flush=True)
assert len(models) == 2
startup = [dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted((OUT / 'startup').glob('*.json'))]
assert len(startup) == 2, 'Actual empty-home app and same-view live refresh evidence are required'
captures = [dict(path=str(p), sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted((OUT / 'captures').glob('*.png'))]
assert len(captures) == 8, 'Four setup and four user Markdown captures are required'
(OUT / 'producer.json').write_text(json.dumps(dict(commands=commands, models=models, pins=pins, captures=captures, startup=startup, operator_input=False, arbitrary_rust_refinement=False), indent=2) + '\n')
