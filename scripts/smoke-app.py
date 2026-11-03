# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Bundled replay plus hidden native-window startup; no credentials or desktop input."""
import json
from pathlib import Path
import plistlib
import subprocess
import sys
import tempfile

ROOT = Path(__file__).resolve().parents[1]


def main():
    bundle, out = (Path(arg).resolve() for arg in sys.argv[1:])
    out.mkdir(parents=True, exist_ok=True)
    (out / 'evidence.json').unlink(missing_ok=True)
    (out / 'native-process.json').unlink(missing_ok=True)
    binary = bundle / 'Contents/MacOS/bboxy450-harness'
    assert bundle.name == 'bboxy450-harness.app', bundle
    plist = plistlib.loads((bundle / 'Contents/Info.plist').read_bytes())
    assert plist['CFBundleName'] == plist['CFBundleDisplayName'] == 'bboxy450-harness'
    assert plist['CFBundleIdentifier'] == 'com.bboxy450.harness'
    subprocess.run(['codesign', '--verify', '--strict', str(bundle)], check=True)
    with tempfile.TemporaryDirectory(prefix='harness-app-', dir='/tmp') as private:
        home = Path(private)
        # No inherited API keys, seat capability, profiles, MCP or developer tools.
        env = {'HOME': private, 'PATH': '/usr/bin:/bin:/usr/sbin:/sbin',
               'TMPDIR': private, 'LANG': 'en_US.UTF-8',
               'BBOXY450_HARNESS_SMOKE_WINDOW': '1'}
        screenshot = out / 'replay.png'
        screenshot.unlink(missing_ok=True)
        subprocess.run([str(binary), 'gui', '--replay',
                        str(ROOT / 'tests/fixtures/gui/rich-all-items.jsonl'),
                        '--headless', '--screenshot', str(screenshot)],
                       env=env, stdin=subprocess.DEVNULL, check=True, timeout=60)
        assert screenshot.stat().st_size > 8
        assert screenshot.read_bytes()[:8] == b'\x89PNG\r\n\x1a\n'
        assert not (home / '.bboxy450-harness').exists(), 'replay accessed live state'
        with (out / 'launcher.log').open('w') as log:
            process = subprocess.Popen([str(bundle / 'Contents/MacOS/launcher')],
                                       cwd=home, env=env, stdin=subprocess.DEVNULL,
                                       stdout=log, stderr=subprocess.STDOUT)
            try:
                native_code = process.wait(timeout=30)
                (out / 'native-process.json').write_text(json.dumps({
                    'pid': process.pid, 'exit_code': native_code,
                    'private_home': True, 'operator_input': False,
                    'window_hidden': True, 'visible_presentation_checked': False}, indent=2) + '\n')
                assert native_code == 0, 'hidden native startup failed'
                text = (out / 'launcher.log').read_text()
                assert 'native window created; surface configured; live frame submitted to private texture; visible=false' in text, text
                sessions = list((home / '.bboxy450-harness/sessions').glob('*.jsonl'))
                assert len(sessions) == 1, 'GUI did not create its own session'
                events = [json.loads(line)['event'] for line in sessions[0].read_text().splitlines()]
                meta = next(event['meta'] for event in events if event['type'] == 'session_started')
                assert meta['cwd'] == private and meta['profile'] == 'nemotron', meta
                assert any(event['type'] == 'item_completed'
                           and event['body']['kind'] == 'notice'
                           and 'profile' in event['body']['text'] for event in events), events
                subprocess.run([str(binary), 'check-transcript', str(sessions[0])],
                               env=env, check=True, timeout=10, stdin=subprocess.DEVNULL)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=5)
        record = {'source': subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=ROOT, text=True).strip(),
                  'private_home': True, 'operator_input': False, 'replay_exit_code': 0,
                  'screenshot': str(screenshot), 'screenshot_bytes': screenshot.stat().st_size,
                  'native_window': 'hidden', 'surface_configured': True,
                  'live_frame_submitted': 'private texture', 'visible_presentation_checked': False,
                  'launcher_pid': process.pid, 'launcher_exit_code': process.returncode,
                  'cwd': meta['cwd'], 'profile': meta['profile'], 'notice': True,
                  'bundle': str(bundle), 'app_name': plist['CFBundleDisplayName'],
                  'codesign_exit_code': 0, 'transcript_exit_code': 0}
        (out / 'evidence.json').write_text(json.dumps(record, indent=2) + '\n')
        print('smoke-app: PASS; bundled PNG, empty-HOME notice, hidden native window/surface/live frame, exit/reap, strict codesign')


if __name__ == '__main__':
    main()
