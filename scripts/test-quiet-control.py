# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Restore production native-record fallbacks; require real replay witnesses to fail."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'target/quiet/controls'


def once(source, old, new):
    if source.count(old) != 1:
        raise ValueError('mutation anchor drift: ' + old)
    return source.replace(old, new)


def gui(source):
    start = source.index('            let (element, rich) = self.face_row(')
    end = source.index('            let mut element = named(', start)
    normal = source[start:end]
    restored = '''            let (element, rich) = if row.kind == ItemKind::Native {
                use bbt_ui::modules::stream_dock::{Entry, Message, StreamDock};
                let dock = StreamDock {
                    dock: false,
                    framed: false,
                    entries: vec![Entry::Message {
                        data: Box::new(Message {
                            from: "Native".into(),
                            role: "system".into(),
                            text: crate::frontends::native_label(&row.body).unwrap_or_default(),
                            time: "Completed".into(),
                            ..Default::default()
                        }),
                        show_channel: Some(false),
                    }],
                };
                (dock.items_at(ctx, index, &self.metrics).remove(0).element, None)
            } else {
''' + normal.replace('            let (element, rich) = ', '                ', 1).rstrip().removesuffix(';') + '''
            };
'''
    return source[:start] + restored + source[end:]


def seat(source):
    source = once(source, '                        | ItemKind::Subagent',
                  '                        | ItemKind::Native\n                        | ItemKind::Subagent')
    source = once(source, 'if matches!(e.kind, ItemKind::Native | ItemKind::Usage)',
                  'if matches!(e.kind, ItemKind::Usage)')
    return once(source, '            (_, Some(Item::Native { .. } | Item::Usage { .. })) => {}',
                '''            (_, Some(body @ Item::Native { .. })) => {
                self.record_line(out, &crate::frontends::native_label(&body).unwrap_or_default());
            }
            (_, Some(Item::Usage { .. })) => {}''')


def rail(source):
    return once(source, '            if !drawn(model, id) {',
                '            if r.kind == ItemKind::Usage {')


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    env = os.environ.copy()
    original = Path.home()
    env['CARGO_HOME'] = env.get('CARGO_HOME', str(original / '.cargo'))
    env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original / '.rustup'))
    for key in ['BBOXY_CAPABILITY', 'BBOXY_CAPABILITY_FILE']:
        env.pop(key, None)
    records = []
    with tempfile.TemporaryDirectory(prefix='quiet-controls-', dir='/tmp') as own:
        env['HOME'] = env['TMPDIR'] = own
        for name, file, mutate, test, assertion in [
            ('gui', 'gui/view.rs', gui, 'gui_render', 'NativeNeverDraws'),
            ('seat', 'seat.rs', seat, 'seat_golden', 'NativeNeverDraws'),
            ('rail', 'gui/face_rows.rs', rail, 'gui_render', 'NativeReplay'),
        ]:
            path = ROOT / 'src/frontends' / file
            original_source = path.read_text()
            digest = hashlib.sha256(path.read_bytes()).hexdigest()
            command = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone',
                       '--test', test, 'native_chunks_do_not_change_replayed_rows',
                       '--', '--exact', '--nocapture']
            try:
                path.write_text(mutate(original_source))
                red = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
                output = red.stdout + red.stderr
                (OUT / f'{name}-red.log').write_text(output)
                if red.returncode != 101 or assertion not in output:
                    raise ValueError(f'{name}: expected exit 101 at {assertion}, got {red.returncode}; see {OUT}')
            finally:
                path.write_text(original_source)
            if hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                raise ValueError('restore hash mismatch: ' + file)
            green = subprocess.run(command, cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
            (OUT / f'{name}-green.log').write_text(green.stdout + green.stderr)
            records.append({'face': name, 'command': command, 'red_exit': red.returncode,
                            'green_exit': green.returncode, 'assertion': assertion,
                            'restored_sha256': digest, 'private_home': True, 'operator_input': False})
            (OUT / 'results.json').write_text(json.dumps(records, indent=2) + '\n')
            print(f'{name}: {red.returncode} -> {green.returncode} at {assertion}', flush=True)
            if green.returncode:
                raise ValueError('restored source failed: ' + name)
    return 0


if __name__ == '__main__':
    raise SystemExit(main())
