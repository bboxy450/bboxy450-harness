# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Broken control for the menu motion contract (harness-spec.md Motion: menu open /
close, menu selection, hover). Zero the menu tween durations in view.rs so every
value snaps to its end state, require tests/gui_menu_motion.rs to go red on the open,
slide, hover and close tests, then restore the source and require green.
Private HOME, no operator input."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/frontends/gui/view.rs'
OUT = ROOT / 'target/motion-b/controls'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone',
           '--test', 'gui_menu_motion', '--', '--nocapture']
DURATIONS = {'MENU_OPEN_MS': '170.', 'MENU_CLOSE_MS': '120.',
             'MENU_SELECT_MS': '120.', 'MENU_HOVER_MS': '150.'}
REGRESSIONS = ['slash_menu_opens_with_a_drop_scale_and_fade_then_sits_idle',
               'slash_selection_slides_between_rows_and_the_tile_tint_crossfades',
               'hover_fills_fade_in_and_out_over_150ms_from_the_pointer',
               'a_closed_menu_fades_out_over_120ms_then_the_clock_is_idle',
               'profile_switcher_drops_in_slides_its_selection_and_fades_out']
# end states are end states whether or not anything tweens
UNCHANGED = ['settled_and_reduced_motion_clocks_paint_end_states_without_frames']


def run(env, name):
    result = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True,
                            text=True, timeout=900)
    output = result.stdout + result.stderr
    (OUT / (name + '.log')).write_text(output)
    return result.returncode, output


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    source = SOURCE.read_text()
    digest = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
    snapped = source
    for name, ms in DURATIONS.items():
        line = f'const {name}: f32 = {ms};'
        if source.count(line) != 1:
            raise ValueError(f'expected exactly one {line}')
        snapped = snapped.replace(line, f'const {name}: f32 = 0.;')
    env = os.environ.copy()
    original_home = Path(env['HOME'])
    env['CARGO_HOME'] = env.get('CARGO_HOME', str(original_home / '.cargo'))
    env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original_home / '.rustup'))
    with tempfile.TemporaryDirectory(prefix='harness-motion-control-', dir='/tmp') as own:
        env['HOME'] = own
        env['TMPDIR'] = own
        try:
            SOURCE.write_text(snapped)
            red, output = run(env, 'snapped')
        finally:
            SOURCE.write_text(source)
        assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == digest
        assert red == 101, f'snapped durations: expected exit 101, got {red}'
        for name in REGRESSIONS:
            assert f'test {name} ... FAILED' in output, name
        for name in UNCHANGED:
            assert f'test {name} ... ok' in output, name
        green, _ = run(env, 'restored')
    record = {'command': COMMAND, 'source': str(SOURCE.relative_to(ROOT)),
              'mutation': {k: '0.' for k in DURATIONS}, 'restored_sha256': digest,
              'snapped_exit': red, 'restored_exit': green,
              'required_failures': REGRESSIONS, 'required_passes': UNCHANGED,
              'private_home': True, 'operator_input': False}
    (OUT / 'results.json').write_text(json.dumps(record, indent=2) + '\n')
    assert green == 0, f'restored source: expected exit 0, got {green}'
    print(f'menu motion control: snapped durations exit 101 ({len(REGRESSIONS)} regressions, '
          f'{len(UNCHANGED)} end-state test still green); restored exit 0')


if __name__ == '__main__':
    main()
