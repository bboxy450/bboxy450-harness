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
reuse_path = os.environ.get('ONBOARDING_REUSE_EVIDENCE')
reused = None
if reuse_path:
    baseline = json.loads(Path(reuse_path).read_text())
    assert baseline['verify']['exit_code'] == 0
    prior = baseline['producer']
    base_head = baseline['producer_head']
    # Only the unchanged installer and its finite graphs are reused. Registry,
    # routing, new-session startup, geometry and every affected compile rerun.
    old = subprocess.check_output(['git', 'show', base_head + ':src/onboarding.rs'], cwd=ROOT, text=True)
    current = (ROOT / 'src/onboarding.rs').read_text()
    for marker, end in [('impl Recipe {', '#[derive(Clone, Debug, PartialEq, Eq)]\npub struct Entry'), ('struct Running {', None)]:
        before = old[old.index(marker):]
        after = current[current.index(marker):]
        if end:
            before, after = before[:before.index(end)], after[:after.index(end)]
        assert before == after, 'Installer changed; run the original focused gate'
    model_inputs = [p for p in baseline['source_hashes'] if p.startswith('crates/bboxy450-harness/') and
                    (p.split('crates/bboxy450-harness/')[1] in ['src/process.rs', 'scripts/formal.py', 'tests/formal_models.rs', 'tests/formal/models.rs', 'tests/formal/export.rs'] or '/verification/' in p)]
    for name in model_inputs:
        path = ROOT / name.removeprefix('crates/bboxy450-harness/')
        assert hashlib.sha256(path.read_bytes()).hexdigest() == baseline['source_hashes'][name], name
    reused_commands = [item for item in prior['commands'] if item['name'] in {'private-discovery-profile-installer-native-gui', 'install-model-and-control'}]
    assert len(reused_commands) == 2
    for item in reused_commands:
        assert item['exit_code'] == 0 and hashlib.sha256(Path(item['log']).read_bytes()).hexdigest() == item['sha256']
    reused = dict(producer_head=base_head, commands=reused_commands,
                  source_hashes={p: baseline['source_hashes'][p] for p in model_inputs},
                  unchanged_installer_sha256=hashlib.sha256(current[current.index('struct Running {'):].encode()).hexdigest())
    run('followup-routing-layout-discovery-new-startup', ['cargo', 'nextest', 'run', *cargo, '--features', 'standalone', '--test', 'harness_onboarding', '-E', 'test(onboarding_) | test(private_discovery_later_addition_and_profile_bytes) | test(private_manager_recipes_are_exact_and_platform_appropriate) | test(private_empty_home_native_app_reaches_setup)'])
else:
    run('private-discovery-profile-installer-native-gui', ['cargo', 'nextest', 'run', *cargo, '--features', 'standalone', '--test', 'harness_onboarding'])
run('followup-markdown-history-pending-new-session', ['cargo', 'nextest', 'run', *cargo, '--features', 'standalone', '--lib', '-E', 'test(onboarding_) | test(gui_polish_pending_requires_causal_durable_user_in_current_birth)'])
if not reused:
    run('install-model-and-control', ['cargo', 'test', *cargo, '--test', 'formal_models', 'onboarding_install_native_model_and_control', '--', '--exact'])
run('strict-producer-and-standalone', ['cargo', 'clippy', *cargo, '--features', 'standalone', '--lib', '--bin', 'bboxy450-harness', '--test', 'harness_onboarding', '--test', 'formal_models', '--no-deps', '--', '-D', 'warnings'])
run('embedded-without-acp', ['cargo', 'check', *cargo, '--no-default-features', '--features', 'embedded', '--lib'])
run('license-headers', ['make', 'test-licenses'])
if reused:
    models, pins = prior['models'], prior['pins']
    assert len(models) == 2 and all(m['graph_equal'] for m in models)
else:
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
(OUT / 'producer.json').write_text(json.dumps(dict(commands=commands, models=models, pins=pins, captures=captures, startup=startup, reused=reused, operator_input=False, arbitrary_rust_refinement=False), indent=2) + '\n')
