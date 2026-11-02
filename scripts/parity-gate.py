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
Exit 1: a regression, a size mismatch, a stale fixture hash or an instrument failure.
Exit 2: no regression, but --target was asked and a target is not met.

The baseline was measured against one web reference (the lab files, hashed). When the lab has
moved since, a regression against the old reference is not measurable: the gate prints
REFERENCE MOVED, skips only the no-regression comparison, still enforces sizes, hashes and
targets, and writes OUT/baseline-candidate.json for the Architect to adopt.

rows_aligned is a diagnostic: the row-structure mismatch after the best global vertical shift
within ±24 px. A large gap between it and the raw score means one offset, not many defects.
"""
import glob, hashlib, json, os, subprocess, sys
from PIL import Image

BASELINE = '.bboxy/runs/parity-baseline-3d76710.json'
TOLERANCE = {'mean_abs_diff': 0.003, 'row_structure_mismatch': 0.02}
CHAT = {'mean_abs_diff': 0.012, 'row_structure_mismatch': 0.08}
RICH = {'mean_abs_diff': 0.030, 'row_structure_mismatch': 0.25}
TARGET = {'acp-claude-acp': CHAT, 'acp-codex-acp': CHAT, 'acp-omp': CHAT, 'acp-pi': CHAT,
          'acp-opencode': RICH, 'glyph-atlas': RICH, 'mermaid': RICH}
LAB_FILES = ['lab.css', 'tokens.js', 'lab.js', 'components.js', 'harness.js']


def lab_hash():
    root = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
    lab = os.environ.get('LAB') or os.path.join(root, '..', 'bboxy450-term', 'lab', 'web')
    h = hashlib.sha256()
    for f in LAB_FILES:
        h.update(open(os.path.join(lab, f), 'rb').read())
    return h.hexdigest()


def ink_rows(im):
    g = im.convert('L'); px = g.load()
    return [sum(1 for x in range(0, g.width, 4) if abs(px[x, y] - px[0, y]) > 24) > 2 for y in range(g.height)]


def rows_aligned(web, nat):
    a, b = ink_rows(web), ink_rows(nat)
    best = (1.0, 0)
    for s in range(-24, 25):
        pairs = [(a[y], b[y + s]) for y in range(len(a)) if 0 <= y + s < len(b)]
        m = sum(x != y for x, y in pairs) / len(a)
        if m < best[0]:
            best = (m, s)
    return best


def main():
    out, want_target = sys.argv[1], '--target' in sys.argv[2:]
    base = json.load(open(BASELINE))
    w, h = base['size']
    moved = base.get('lab_sha256') != lab_hash()
    if moved:
        print(f'REFERENCE MOVED: lab files differ from the baseline ({base.get("lab_sha256", "?")[:12]}); '
              'no-regression comparison skipped, targets still enforced')
    fixtures = sorted(glob.glob('tests/fixtures/parity/*.jsonl'))
    names = {os.path.basename(f)[:-6] for f in fixtures}
    bad, missed, candidate = [], [], {}
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
        web = Image.open(os.path.join(d, 'web.png')); nat = Image.open(os.path.join(d, 'native.png'))
        for png, im in (('web.png', web), ('native.png', nat)):
            if im.size != (w, h):
                bad.append(f'{n}: {png} is {im.size[0]}x{im.size[1]}, not {w}x{h}')
        score = json.load(open(os.path.join(d, 'score.json')))
        aligned, shift = rows_aligned(web, nat)
        candidate[n] = {'sha256': b['sha256'], **{k: score[k] for k in TOLERANCE}}
        for k, tol in TOLERANCE.items():
            v = score[k]
            if not moved and v > b[k] + tol:
                bad.append(f'{n}: {k} {v} regressed past baseline {b[k]} + {tol}')
            if v > TARGET[n][k]:
                missed.append(f'{n}: {k} {v} > target {TARGET[n][k]}')
            print(f'{n} {k} {v} baseline {b[k]} target {TARGET[n][k]}')
        print(f'{n} rows_aligned {aligned:.4f} at shift {shift:+d}px (diagnostic)')
    if moved:
        json.dump({**base, 'lab_sha256': lab_hash(), 'fixtures': candidate},
                  open(os.path.join(out, 'baseline-candidate.json'), 'w'), indent=1)
    for line in bad: print('FAIL', line)
    for line in missed: print('TARGET', line)
    if bad: return 1
    if want_target and missed: return 2
    print('parity-gate: no regression' + (', targets met' if want_target else '') + (' (reference moved)' if moved else ''))
    return 0


if __name__ == '__main__':
    sys.exit(main())
