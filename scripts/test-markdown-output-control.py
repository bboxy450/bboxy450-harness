# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""Disable the production tool Markdown route, require regressions, restore and pass."""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import tempfile

ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/frontends/gui/face_rows.rs'
OUT = ROOT / 'target/markdown-out/control'
COMMAND = ['cargo', 'test', '--offline', '--locked', '--features', 'standalone',
           '--test', 'gui_rich', 'markdown_output_', '--', '--nocapture']
REGRESSIONS = [
    'markdown_output_read_renders_headings_lists_and_scrolls_without_reparsing',
    'markdown_output_mermaid_tool_result_is_a_native_figure_and_execute_stays_literal',
]


def run(env, name):
    result = subprocess.run(COMMAND, cwd=ROOT, env=env, capture_output=True,
                            text=True, timeout=600)
    output = result.stdout + result.stderr
    (OUT / (name + '.log')).write_text(output)
    return result.returncode, output


def main():
    OUT.mkdir(parents=True, exist_ok=True)
    source = SOURCE.read_text()
    digest = hashlib.sha256(SOURCE.read_bytes()).hexdigest()
    route = 'super::super::rich::tool_markdown(d.kind, &d.target, failed, text)'
    assert source.count(route) == 1, 'expected exactly one production tool Markdown route'
    env = os.environ.copy()
    original = Path(env['HOME'])
    env['CARGO_HOME'] = env.get('CARGO_HOME', str(original / '.cargo'))
    env['RUSTUP_HOME'] = env.get('RUSTUP_HOME', str(original / '.rustup'))
    env.pop('BBOXY_CAPABILITY', None)
    env.pop('BBOXY_CAPABILITY_FILE', None)
    with tempfile.TemporaryDirectory(prefix='harness-markdown-control-', dir='/tmp') as own:
        env['HOME'] = own
        env['TMPDIR'] = own
        try:
            SOURCE.write_text(source.replace(route, 'None::<String>'))
            red, output = run(env, 'disabled-route')
        finally:
            SOURCE.write_text(source)
        assert hashlib.sha256(SOURCE.read_bytes()).hexdigest() == digest
        assert red == 101, f'disabled route: expected exit 101, got {red}'
        for name in REGRESSIONS:
            assert f'test {name} ... FAILED' in output, name
        green, _ = run(env, 'restored')
    record = {'command': COMMAND, 'source': str(SOURCE.relative_to(ROOT)),
              'restored_sha256': digest, 'disabled_route_exit': red,
              'restored_exit': green, 'required_failures': REGRESSIONS,
              'private_home': True, 'operator_input': False}
    (OUT / 'results.json').write_text(json.dumps(record, indent=2) + '\n')
    assert green == 0, f'restored route: expected exit 0, got {green}'
    print('Markdown output control: disabled route exit 101 (Read + Mermaid); restored exit 0')


if __name__ == '__main__':
    main()
