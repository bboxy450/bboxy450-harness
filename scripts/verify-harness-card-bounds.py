#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Rendering-only gate. Retain signed source-bound core proof; never rerun it."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
ROOT = Path(__file__).resolve().parents[1]
TARGET = Path(os.environ.get('TARGET_DIR', 'target')).resolve()
OUT = TARGET / 'harness-card-bounds'
OUT.mkdir(parents=True, exist_ok=True)
CORE = '9501b79845cd7486738e6bf948a09813e70d87cf'
RENDER = 'a832bd6ff4b31a36064508632c47840a58cfe56c'

def sha(data): return hashlib.sha256(data).hexdigest()
def blob(commit, path):
    return subprocess.check_output(['git', 'show', f'{commit}:{path}'], cwd=ROOT)

core_bytes = blob(CORE, '.bboxy/runs/codex-acp-controls.json')
assert sha(core_bytes) == 'f46be4ea94b79fbe57bf8a1a8917b0482ef2996a59318d008e49097c5c1310b2'
core = json.loads(core_bytes)
assert core['verify'] == dict(command='make verify-acp-controls', exit_code=0)
assert len(core['commands']) == 15 and all(c['exit_code'] == 0 for c in core['commands'])
paths = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', CORE], cwd=ROOT, text=True).splitlines()
semantic = [p for p in paths if
    (p.startswith('src/') and not p.startswith('src/frontends/gui/')) or
    p in {'src/frontends/gui/model.rs', 'src/frontends/gui/bridge.rs', 'tests/acp_controls.rs',
          'tests/acp_agent.rs', 'tests/acp_launch_context.rs', 'scripts/formal.py'} or
    p.startswith(('tests/fixtures/', 'tests/formal', 'verification/'))]
for path in semantic:
    assert (ROOT/path).read_bytes() == blob(CORE, path), f'semantic input changed: {path}; core proof cannot be reused'
semantic_hashes = {p: sha((ROOT/p).read_bytes()) for p in semantic}
render_bytes = blob(RENDER, '.bboxy/runs/codex-acp-controls.json')
assert sha(render_bytes) == '25a795b1a2e5d88e9008a4c935ea23039c8911f46f92fc3ecf7f5c13ec39ab5c'
render = json.loads(render_bytes)
# The inherited checkpoint is retained exactly. Make/credit changes receive this fresh proof.
for path, digest in render['source_hashes'].items():
    if path not in {'Makefile', 'ATTRIBUTION.md'}:
        assert sha((ROOT/path).read_bytes()) == digest, f'inherited verified input changed: {path}'
print(f'card-bounds: retained original core proof; {len(semantic)} semantic inputs unchanged', flush=True)
files = sorted(set(semantic + list(render['source_hashes']) +
    ['Makefile', 'ATTRIBUTION.md', 'tests/gui_card_bounds.rs', 'scripts/verify-harness-card-bounds.py']))
source_hashes = {p: sha((ROOT/p).read_bytes()) for p in files}
env = dict(os.environ, RUSTC_WRAPPER='', CARGO_BUILD_JOBS='2', RUST_TEST_THREADS='2',
    CLIPPY_CONF_DIR=str(ROOT/'src/frontends/gui/lint'), HARNESS_CARD_BOUNDS_CAPTURE_DIR=str(OUT/'captures'))
cargo = ['cargo', '--offline', '--locked', '--jobs', '2', '--target-dir', str(TARGET)]
records = []
def run(name, command):
    log = OUT/(name+'.log')
    with log.open('w') as stream:
        result = subprocess.run(command, cwd=ROOT, env=env, stdout=stream, stderr=subprocess.STDOUT)
    records.append(dict(name=name, command=command, exit_code=result.returncode, log=str(log), sha256=sha(log.read_bytes())))
    print(f'card-bounds: {name}: exit {result.returncode}', flush=True)
    if result.returncode:
        print(log.read_text()[-6000:], file=sys.stderr)
        raise SystemExit(result.returncode)

def cmd(verb, *args): return [cargo[0], verb, *cargo[1:], *args]
run('native-card-geometry-raster-and-control', cmd('test', '--features', 'embedded', '--test', 'gui_card_bounds',
    '--test', 'gui_rich', 'acp_controls_tool_cards', '--', '--nocapture'))
run('strict-embedded-compile', cmd('clippy', '--no-default-features', '--features', 'embedded', '--lib', '--no-deps', '--', '-D', 'warnings'))
run('named-acp-native-options', cmd('test', '--features', 'embedded', '--test', 'acp_controls',
    'acp_controls_native_popover_routes_real_selection_and_unavailable_labels', '--', '--nocapture'))
assert source_hashes == {p: sha((ROOT/p).read_bytes()) for p in files}, 'inputs changed during focused verification'
evidence = dict(verify=dict(command='make verify-harness-card-bounds', exit_code=0),
    source_head_at_check=subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
    fresh_scope='Three named rendering/strict-embedded/ACP-native commands only; no core/formal/vendor gate rerun.',
    commands=records, source_hashes=source_hashes,
    retained_core=dict(commit=CORE, receipt_sha256=sha(core_bytes), semantic_input_hashes=semantic_hashes,
        original_command_count=15, original_logs='Independently verified by Oracle per amended task; retained, not reconstructed or relabeled as fresh.',
        original_verify=core['verify']),
    retained_rendering=dict(commit=RENDER, receipt_sha256=sha(render_bytes),
        scope='Exact inherited renderer and named input hashes retained; earlier loader/history/formal results remain earlier evidence.'),
    captures=[dict(path=str(p),sha256=sha(p.read_bytes())) for p in sorted((OUT/'captures').glob('*.png'))],
    operator_input=False, omitted=['old ACP/formal gate','whole suites','paid/live providers','desktop input','install','pin change','integration'])
(ROOT/'.bboxy/runs/codex-acp-controls-card-bounds.json').write_text(json.dumps(evidence, indent=2)+'\n')
