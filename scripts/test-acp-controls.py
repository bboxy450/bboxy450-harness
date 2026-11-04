#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Focused ACP controls gate; exact named native/process/raster contracts, no vendor calls."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get('TARGET_DIR', 'target')).resolve()
OUT = TARGET / 'acp-controls'
OUT.mkdir(parents=True, exist_ok=True)
env = dict(os.environ, RUSTC_WRAPPER='', CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='2',
           CLIPPY_CONF_DIR=str(ROOT/'src/frontends/gui/lint'), ACP_CONTROLS_FORMAL_OUT=str(OUT / 'formal'), GUI_POLISH_FORMAL_OUT=str(OUT / 'pending-formal'),
           PROFILES_PENDING_CAPTURE_DIR=str(OUT / 'captures'))
records = []

def run(name, command):
    log = OUT / (name + '.log')
    with log.open('w') as stream:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    records.append(dict(command=command, exit_code=result.returncode, log=str(log.relative_to(ROOT)), sha256=hashlib.sha256(log.read_bytes()).hexdigest()))
    print(f'acp-controls: {name}: exit {result.returncode}', flush=True)
    if result.returncode:
        print(log.read_text()[-7000:], file=sys.stderr)
        raise SystemExit(result.returncode)

files = ['Makefile', 'Cargo.toml', 'Cargo.lock', 'ATTRIBUTION.md', 'protocol.schema.json', 'scripts/test-acp-controls.py', 'scripts/formal.py',
         'verification/contracts.toml', 'verification/corpus.toml', 'verification/coverage.toml', 'verification/tools.lock.json',
         'tests/acp_controls.rs', 'tests/acp_agent.rs', 'tests/acp_launch_context.rs', 'tests/gui_render.rs', 'tests/gui_bridge.rs', 'tests/gui_rich.rs',
         'tests/formal/models.rs', 'tests/formal_models.rs', 'tests/fixtures/acp-agent/config_agent.py']
files += [str(p.relative_to(ROOT)) for p in (ROOT/'src').rglob('*.rs')]
hashes = {p: hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in sorted(set(files))}
cargo = ['cargo', '--offline', '--locked', '--jobs', '2', '--target-dir', str(TARGET)]

def build(command, *args): return [cargo[0], command, *cargo[1:], *args]
run('default', build('check', '--lib'))
run('embedded', build('check', '--no-default-features', '--features', 'embedded', '--lib'))
run('acp-standalone', build('check', '--features', 'acp,standalone', '--lib', '--bin', 'bboxy450-harness'))
run('process-and-native-controls', build('test', '--features', 'embedded', '--test', 'acp_controls', '--', '--nocapture'))
run('existing-acp', build('test', '--test', 'acp_agent', '--test', 'acp_launch_context'))
run('bridge', build('test', '--features', 'embedded', '--test', 'gui_bridge'))
run('pending-core', build('test', '--features', 'embedded', '--lib', 'gui_polish_pending'))
run('pending-raster', build('test', '--features', 'embedded', '--test', 'gui_render', 'profiles_pending', '--', '--nocapture'))
run('preserved-gui', build('test', '--features', 'embedded', '--test', 'gui_render', '--test', 'gui_rich', 'gui_polish'))
run('path-unit', build('test', '--features', 'standalone', '--lib', 'acp_controls_standalone'))
run('embedded-lint', build('clippy', '--no-default-features', '--features', 'embedded', '--lib', '--no-deps', '--', '-D', 'warnings'))
run('all-lint', build('clippy', '--all-features', '--lib', '--bin', 'bboxy450-harness', '--no-deps', '--', '-D', 'warnings'))
run('schema', build('test', '--test', 'protocol_golden'))
run('formal-controls', build('test', '--test', 'formal_models', 'acp_controls_native'))
run('formal-pending', build('test', '--test', 'formal_models', 'gui_polish_pending'))
# Only the two affected finite families. Reuse the canonical pinned graph/verdict checker.
spec = importlib.util.spec_from_file_location('formal', ROOT/'scripts/formal.py')
formal = importlib.util.module_from_spec(spec)
spec.loader.exec_module(formal)
java, jar, pins = formal.tools(TARGET/'formal')
models = []
expected_names = {'HarnessAcpControls', 'BrokenHarnessAcpControlsStale', 'BrokenHarnessAcpControlsReplay',
                  'HarnessGuiPendingPrompt', 'BrokenHarnessGuiPendingPromptAck', 'BrokenHarnessGuiPendingPromptStale'}
for model_root in [OUT/'formal', OUT/'pending-formal']:
    for line in (model_root/'manifest.tsv').read_text().splitlines():
        name, expected, count, width = line.split('\t')
        assert name in expected_names, 'unexpected focused model'
        graph = formal.tlc(java, jar, model_root, name, True)
        assert formal.verdict(graph) == 'pass'
        actual = formal.dot_graph(model_root/f'{name}.dot', int(width))
        assert actual == formal.native_graph(model_root, name) and len(actual[0]) == int(count), f'full graph mismatch: {name}'
        checked = formal.tlc(java, jar, model_root, name, False)
        assert formal.verdict(checked) == expected, f'verdict disagreement: {name}'
        models.append(dict(name=name, expected=expected, states=len(actual[0]), edges=len(actual[1]), graph_equal=True,
                           graph_exit=graph.returncode, tlc_exit=checked.returncode, tla_sha256=hashlib.sha256((model_root/f'{name}.tla').read_bytes()).hexdigest()))
        print(f'acp-controls: {name}: {expected}; {len(actual[0])} states, {len(actual[1])} transitions match', flush=True)
assert {m['name'] for m in models} == expected_names
assert hashes == {p: hashlib.sha256((ROOT/p).read_bytes()).hexdigest() for p in hashes}, 'source changed during verification'
evidence = dict(verify=dict(command='make verify-acp-controls', exit_code=0),
    source_head_at_check=subprocess.check_output(['git','rev-parse','HEAD'],cwd=ROOT,text=True).strip(),
    source_hashes=hashes, commands=records, models=models, pins=pins, operator_input=False,
    captures=[dict(path=str(p.relative_to(ROOT)),sha256=hashlib.sha256(p.read_bytes()).hexdigest()) for p in sorted((OUT/'captures').glob('*.png'))],
    scope='Affected default/embedded/ACP compile; named private ACP processes, Unix reconnect, GUI controls and preserved raster witnesses; two finite native/reference model families only.',
    omitted=['whole test/formal/parity/runtime corpora', 'paid/live vendor probes', 'OS input', 'install', 'push', 'shared branch merge'],
    complete_tlc_replacement=False, rust_refinement_proven=False)
(ROOT/'.bboxy/runs/codex-acp-controls.json').write_text(json.dumps(evidence,indent=2)+'\n')
