# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Local, unattended source-header and credential-shape gates."""
from pathlib import Path
import re
import os
import subprocess
import sys
ROOT=Path(__file__).resolve().parents[1]
HEADER=(ROOT / 'ATTRIBUTION.md').read_text().split('```\n')[1].split('```')[0].strip().splitlines()
def paths():
    for name in ['src','tests','scripts','verification','.bboxy']:
        for path in (ROOT/name).rglob('*'):
            if path.is_file() and '__pycache__' not in path.parts and 'evidence' not in path.parts:
                yield path

def headers():
    for path in list(paths())+[ROOT/'Cargo.toml']:
        toml=path==ROOT/'Cargo.toml' or path.parent==ROOT/'verification' and path.suffix=='.toml'
        if not toml and path.suffix not in {'.rs','.py','.sh'}:continue
        lines=path.read_text().splitlines()
        if lines and lines[0].startswith('#!/') :lines=lines[1:]
        expected=HEADER if path.suffix=='.rs' else [l.replace('//','#',1) for l in HEADER]
        if lines[:6]!=expected:raise ValueError('source header differs: '+str(path.relative_to(ROOT)))
    print('headers: PASS')
def secrets():
    patterns=[re.compile(r'sk-(?:or-v1-)?[A-Za-z0-9]{20,}'),re.compile(r'(?i)(?:api_key|key)\s*[:=]\s*[\x22\x27]?[0-9a-f]{32,}')]
    tracked=subprocess.check_output(['git','ls-files','-z'],cwd=ROOT).split(b'\0')
    for name in tracked:
        if not name:continue
        path=ROOT/os.fsdecode(name)
        if not path.exists() and not path.is_symlink():raise ValueError('tracked file missing: '+str(path.relative_to(ROOT)))
        content=os.readlink(path) if path.is_symlink() else path.read_bytes().decode('latin-1')
        if any(p.search(content) for p in patterns):raise ValueError('credential shape in '+str(path.relative_to(ROOT))+' (value suppressed)')
    print('secret scan: PASS')
def kb_records(root=ROOT/'.bboxy'):
    """Every .bboxy/**/*.md is an index record: frontmatter with a unique id and a positive integer number."""
    ids,numbers={},{}
    for path in sorted(root.rglob('*.md')):
        name=str(path.relative_to(root.parent))
        lines=path.read_text(errors='replace').splitlines()
        if not lines or lines[0]!='---' or '---' not in lines[1:]:raise ValueError('kb record without frontmatter: '+name)
        front=lines[1:lines.index('---',1)]
        def field(key):
            found=[l.split(':',1)[1].strip().strip('"\'') for l in front if re.match(key+r':\s*\S',l)]
            if not found:raise ValueError('kb record without '+key+': '+name)
            return found[0]
        record,number=field('id'),field('number')
        if not number.isdigit() or int(number)<1:raise ValueError('kb record number is not a positive integer: '+name)
        for seen,key in ((ids,record),(numbers,number)):
            if key in seen:raise ValueError('kb record '+('id' if seen is ids else 'number')+' '+key+' repeats: '+seen[key]+', '+name)
            seen[key]=name
    print('kb records: PASS ('+str(len(numbers))+')')
def kb_records_control():
    """Broken control: a record without a number must fail the check."""
    import tempfile
    with tempfile.TemporaryDirectory() as tmp:
        bad=Path(tmp)/'.bboxy'/'runs'
        bad.mkdir(parents=True)
        (bad/'report.md').write_text('# a run report with no frontmatter\n')
        try:kb_records(Path(tmp)/'.bboxy')
        except ValueError:print('kb records control: PASS (rejected)');return
    raise ValueError('kb records control: a record without frontmatter was accepted')
if __name__=='__main__':
    try:{'headers':headers,'secrets':secrets,'kb-records':kb_records,'kb-records-control':kb_records_control}[sys.argv[1]]()
    except (ValueError,KeyError) as error:print(error,file=sys.stderr);sys.exit(1)
