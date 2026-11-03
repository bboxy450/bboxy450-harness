# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

"""R15: one copy of each toolkit crate, including the UI-thread registry."""
import json
from pathlib import Path
import subprocess
root=Path(__file__).resolve().parents[1]
result=subprocess.run(['cargo','metadata','--offline','--locked','--format-version','1','--features','standalone'],cwd=root,check=True,text=True,capture_output=True)
packages=json.loads(result.stdout)['packages']
by_name={}
for package in packages:
    if package['name'].startswith('bboxy450-'):
        by_name.setdefault(package['name'],[]).append(package)
assert all(len(entries)==1 for entries in by_name.values()), {name:[p['source'] for p in entries] for name,entries in by_name.items() if len(entries)>1}
for name in ['bboxy450-ui','bboxy450-doc','bboxy450-render','bboxy450-ui-core','bboxy450-rt']:
    assert len(by_name[name])==1, name
toolkit_names = ['bboxy450-ui', 'bboxy450-doc', 'bboxy450-render', 'bboxy450-ui-core', 'bboxy450-mermaid']
toolkit_sources = {by_name[name][0]['source'] for name in toolkit_names}
term_source = 'git+https://github.com/bboxy450/bboxy450-term?rev='
# Term's embedded workspace patches this same toolkit to local paths (source=None).
assert toolkit_sources == {None} or (len(toolkit_sources) == 1 and next(iter(toolkit_sources)).startswith(term_source)), toolkit_sources
assert {p['source'] for p in packages if (p['source'] or '').startswith(term_source)} == toolkit_sources - {None}
print('gui-dependencies: PASS; one toolkit source including Mermaid, one copy of each package, shared UI-thread registry')
