# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Restore main's missing title-bar asset; require native startup to reject it."""
import hashlib
import json
from pathlib import Path
import shutil
import subprocess

ROOT = Path(__file__).resolve().parents[1]
OUT = ROOT / 'target/startup-control'
SOURCE = ROOT / 'src/frontends/gui/assets.rs'


def run(label):
    command = ['make', 'smoke-app']
    smoke = ROOT / 'target/app-smoke'
    # A failed build cannot borrow process evidence from an earlier attempt.
    for name in ['launcher.log', 'native-process.json', 'evidence.json']:
        (smoke / name).unlink(missing_ok=True)
    with (OUT / (label + '.log')).open('w') as log:
        result = subprocess.run(command, cwd=ROOT, stdout=log,
                                stderr=subprocess.STDOUT, timeout=300)
    smoke = ROOT / 'target/app-smoke'
    for name in ['launcher.log', 'native-process.json', 'evidence.json']:
        path = smoke / name
        if path.exists():
            shutil.copyfile(path, OUT / (label + '-' + name))
    return {'command': command, 'exit_code': result.returncode}


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    original = SOURCE.read_bytes()
    text = original.decode()
    needle = '[MARK, "bboxy"]'
    assert text.count(needle) == 1, 'startup-control asset anchor drift'
    # This is the actual 0c40316 registration defect: the face mark was present,
    # but the shared TitleBar asks for "bboxy" and the renderer refuses that frame.
    try:
        SOURCE.write_text(text.replace(needle, '[MARK]'))
        red = run('missing-titlebar-asset')
    finally:
        SOURCE.write_bytes(original)
    assert SOURCE.read_bytes() == original
    green = run('restored')
    native = json.loads((OUT / 'missing-titlebar-asset-native-process.json').read_text())
    message = (OUT / 'missing-titlebar-asset-launcher.log').read_text()
    assert red['exit_code'] != 0 and native['exit_code'] == 1, (red, native, message)
    assert 'unregistered bitmap' in message, message
    assert green['exit_code'] == 0, green
    record = {'source_sha256': hashlib.sha256(original).hexdigest(),
              'restored': SOURCE.read_bytes() == original,
              'negative': red, 'native_negative': native, 'positive': green,
              'private_home': True, 'operator_input': False}
    (OUT / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n')
    print('startup-control: PASS; missing title-bar image exits 1, restored native startup exits 0')


if __name__ == '__main__':
    main()
