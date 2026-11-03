# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Broken controls for the standalone window chrome (harness-spec.md Standalone window and
the composer's alive rim). Two production mutations, each must turn the named
tests/gui_chrome.rs tests red, then the restored source must be green:

  1. the zoom clamp: MAX becomes 1000 in src/frontends/gui/zoom.rs, so Cmd+= walks past 160 %;
  2. the rim activity mapping: src/frontends/gui/face.rs rim_activity answers Idle for every
     phase, so a running turn and a waiting approval lose their rim colour.

Private HOME, no operator input."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'target/chrome/controls'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone',
           '--test', 'gui_chrome', '--', '--nocapture']
MUTATIONS = [
    {
        'name': 'zoom-clamp',
        'source': ROOT / 'src/frontends/gui/zoom.rs',
        'before': 'pub const MAX: u8 = 160;',
        'after': 'pub const MAX: u8 = 250;',
        'regressions': ['zoom_steps_and_clamps_on_the_ten_percent_grid'],
        'unchanged': ['rim_activity_follows_run_ask_error_and_idle',
                      'title_bar_is_drawn_with_the_crumb_and_hits_follow_the_tree'],
    },
    {
        'name': 'rim-activity',
        'source': ROOT / 'src/frontends/gui/face.rs',
        'before': 'pub fn rim_activity(live: bool, ask: bool, errored: bool) -> ui::GlowActivity {\n    if ask {',
        'after': 'pub fn rim_activity(live: bool, ask: bool, errored: bool) -> ui::GlowActivity {\n    if false && (live || ask || errored) {',
        'regressions': ['rim_activity_follows_run_ask_error_and_idle',
                        'composer_rim_reads_the_session'],
        'unchanged': ['zoom_steps_and_clamps_on_the_ten_percent_grid',
                      'title_bar_is_drawn_with_the_crumb_and_hits_follow_the_tree'],
    },
]


def run(env, name):
    result = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True,
                            text=True, timeout=1200)
    output = result.stdout + result.stderr
    (OUT / (name + '.log')).write_text(output)
    return result.returncode, output


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    original_home = Path(env['HOME'])
    env['CARGO_HOME'] = env.get('CARGO_HOME', str(original_home / '.cargo'))
    env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original_home / '.rustup'))
    records = []
    with tempfile.TemporaryDirectory(prefix='harness-chrome-control-', dir='/tmp') as own:
        env['HOME'] = own
        env['TMPDIR'] = own
        for mutation in MUTATIONS:
            source_path = mutation['source']
            source = source_path.read_text()
            digest = hashlib.sha256(source_path.read_bytes()).hexdigest()
            if source.count(mutation['before']) != 1:
                raise ValueError(f"expected exactly one {mutation['before']!r} in {source_path}")
            try:
                source_path.write_text(source.replace(mutation['before'], mutation['after']))
                red, output = run(env, mutation['name'] + '-broken')
            finally:
                source_path.write_text(source)
            assert hashlib.sha256(source_path.read_bytes()).hexdigest() == digest
            assert red == 101, f"{mutation['name']}: expected exit 101, got {red}"
            for name in mutation['regressions']:
                assert f'test {name} ... FAILED' in output, (mutation['name'], name)
            for name in mutation['unchanged']:
                assert f'test {name} ... ok' in output, (mutation['name'], name)
            records.append({'mutation': mutation['name'], 'source': str(source_path.relative_to(ROOT)),
                            'before': mutation['before'], 'after': mutation['after'],
                            'restored_sha256': digest, 'broken_exit': red,
                            'required_failures': mutation['regressions'],
                            'required_passes': mutation['unchanged']})
        green, _ = run(env, 'restored')
    record = {'command': COMMAND, 'mutations': records, 'restored_exit': green,
              'private_home': True, 'operator_input': False}
    (OUT / 'results.json').write_text(json.dumps(record, indent=2) + '\n')
    assert green == 0, f'restored source: expected exit 0, got {green}'
    print(f'chrome control: {len(records)} mutations each exit 101 with their regressions; restored exit 0')


if __name__ == '__main__':
    main()
