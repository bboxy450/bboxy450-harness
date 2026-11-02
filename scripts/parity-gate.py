# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Rich-face parity gate (verification matrix R27): runs scripts/parity-web.sh on every
tests/fixtures/parity fixture and turns the scores into an exit code.

  python3 -I scripts/parity-gate.py OUT [--target]     env LAB=<bboxy450-term>/lab/web BIN=...

Exit 0: every fixture matches its baseline hash, both PNGs are 900x1100 and no score is worse
than baseline + tolerance. With --target, every score must also meet its target.
Exit 1: a regression, a size mismatch, a stale baseline or an instrument failure.
Exit 2: no regression, but --target was asked and a target is not met.
"""
import glob, hashlib, json, os, subprocess, sys
from PIL import Image

BASELINE = '.bboxy/runs/parity-baseline-3d76710.json'
TOLERANCE = {'mean_abs_diff': 0.003, 'row_structure_mismatch': 0.02}
CHAT = {'mean_abs_diff': 0.012, 'row_structure_mismatch': 0.08}
RICH = {'mean_abs_diff': 0.030, 'row_structure_mismatch': 0.25}
TARGET = {'acp-claude-acp': CHAT, 'acp-codex-acp': CHAT, 'acp-omp': CHAT, 'acp-pi': CHAT,
          'acp-opencode': RICH, 'glyph-atlas': RICH, 'mermaid': RICH}

def main():
    out, want_target = sys.argv[1], '--target' in sys.argv[2:]
    base = json.load(open(BASELINE))
    w, h = base['size']
    fixtures = sorted(glob.glob('tests/fixtures/parity/*.jsonl'))
    names = {os.path.basename(f)[:-6] for f in fixtures}
    bad, missed = [], []
    if names != set(base['fixtures']) or names != set(TARGET):
        bad.append(f'fixture set {sorted(names)} differs from baseline/targets')
    for f in fixtures:
        n = os.path.basename(f)[:-6]
        b = base['fixtures'].get(n)
        if b is None or hashlib.sha256(open(f, 'rb').read()).hexdigest() != b['sha256']:
            bad.append(f'{n}: fixture hash differs from baseline (re-baseline by Architect decision)')
            continue
        d = os.path.join(out, n)
        r = subprocess.run(['sh', 'scripts/parity-web.sh', f, d, f'{w}x{h}'], capture_output=True, text=True)
        if r.returncode != 0:
            bad.append(f'{n}: parity-web.sh exited {r.returncode}: {r.stderr.strip()[-300:]}')
            continue
        for png in ('web.png', 'native.png'):
            size = Image.open(os.path.join(d, png)).size
            if size != (w, h):
                bad.append(f'{n}: {png} is {size[0]}x{size[1]}, not {w}x{h}')
        score = json.load(open(os.path.join(d, 'score.json')))
        for k, tol in TOLERANCE.items():
            v = score[k]
            if v > b[k] + tol:
                bad.append(f'{n}: {k} {v} regressed past baseline {b[k]} + {tol}')
            if v > TARGET[n][k]:
                missed.append(f'{n}: {k} {v} > target {TARGET[n][k]}')
            print(f'{n} {k} {v} baseline {b[k]} target {TARGET[n][k]}')
    for line in bad: print('FAIL', line)
    for line in missed: print('TARGET', line)
    if bad: return 1
    if want_target and missed: return 2
    print('parity-gate: no regression' + (', targets met' if want_target else ''))
    return 0

if __name__ == '__main__':
    sys.exit(main())
