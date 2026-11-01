# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Break real rendering, demand the named structural assertion, then restore it."""
import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / '.bboxy/runs/e3c-controls'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone',
           '--test', 'gui_render', 'structural_layout_smoke_checks_restored_geometry',
           '--', '--exact', '--nocapture']

def once(source, old, new):
    if source.count(old) != 1:
        raise ValueError('mutation anchor drift: ' + old)
    return source.replace(old, new)

def main():
    cases = [
        ('markdown', 'rich.rs', 'MarkdownBlocksInsideCard',
         'translate(&mut solved, dx, dy);', 'translate(&mut solved, dx + 2048., dy);'),
        ('header-fill', 'view.rs', 'AccordionHeaderFillCoversRow',
         'header.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Surface2));',
         'header.style.overrides.fill = Some(ui::Paint::solid(ui::Color::Transparent));'),
        ('inline', 'view.rs', 'InlineControlsInsideRow',
         '.expect("inline control style"),',
         '.map(|mut e| {if let c::Element::Box(b) = &mut e {b.style.overrides.width = Some(ui::Length::Px(2048.));} e}).expect("inline control style"),'),
        ('empty-box', 'view.rs', 'NoEmptyPaintedBoxes',
         'key: None,\n                }\n                .element(ctx)\n                .expect("approval button style");',
         'key: Some(String::new()),\n                }\n                .element(ctx)\n                .expect("approval button style");'),
    ]
    OUT.mkdir(parents=True, exist_ok=True)
    records = []
    env = os.environ.copy()
    original_home = Path(env['HOME'])
    env['CARGO_HOME'] = env.get('CARGO_HOME', str(original_home / '.cargo'))
    env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original_home / '.rustup'))
    env.pop('BBOXY_CAPABILITY', None)
    with tempfile.TemporaryDirectory(prefix='gui-polish-controls-', dir='/tmp') as own:
        env['HOME'] = own
        env['TMPDIR'] = own
        for name, file, assertion, old, new in cases:
            path = ROOT / 'src/frontends/gui' / file
            source = path.read_text()
            digest = hashlib.sha256(source.encode()).hexdigest()
            try:
                path.write_text(once(source, old, new))
                red = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
                output = red.stdout + red.stderr
                (OUT / f'{name}-red.log').write_text(output)
                if red.returncode != 101 or assertion not in output:
                    raise ValueError(f'{name}: wanted exit 101 at {assertion}, got {red.returncode}; inspect red log')
            finally:
                path.write_text(source)
            if hashlib.sha256(path.read_bytes()).hexdigest() != digest:
                raise ValueError('restore hash mismatch: ' + file)
            green = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True, text=True, timeout=600)
            (OUT / f'{name}-green.log').write_text(green.stdout + green.stderr)
            records.append({'control': name, 'source': str(path.relative_to(ROOT)),
                            'assertion': assertion, 'command': COMMAND,
                            'red_exit': red.returncode, 'green_exit': green.returncode,
                            'restored_sha256': digest, 'private_home': True, 'operator_input': False})
            (OUT / 'results.json').write_text(json.dumps(records, indent=2) + '\n')
            print(f'{name}: {red.returncode} -> {green.returncode} at {assertion}', flush=True)
            if green.returncode:
                raise ValueError('restored source failed: ' + name)
    return 0

if __name__ == '__main__':
    claim = ROOT / 'target/gui-polish-controls.claim'
    claim.parent.mkdir(parents=True, exist_ok=True)
    with claim.open('a+') as handle:
        try:
            fcntl.flock(handle, fcntl.LOCK_EX | fcntl.LOCK_NB)
        except BlockingIOError:
            raise SystemExit('another mutation pass owns this worktree')
        handle.seek(0)
        handle.truncate()
        handle.write(str(os.getpid()) + '\n')
        handle.flush()
        raise SystemExit(main())
