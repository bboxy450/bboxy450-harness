# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Bboxy log IR evolution gate (.bboxy/specs/log-ir-v1.md §2): only add, never change or remove.

  python3 -I scripts/ir-compat.py check      protocol.schema.json against verification/protocol-v1.lock.json
  python3 -I scripts/ir-compat.py control    each forbidden mutation of the current schema must be refused
  python3 -I scripts/ir-compat.py lock       rewrite the lock (an Architect act: record the reason in the spec)

The surface is flattened from the schemars export: every definition, every field with its type
signature and required flag, every variant of a tagged union or string enum. Allowed: a new
definition, a new optional field, a new variant. Refused: a removed definition, field or variant;
a changed type, tag key or definition shape; a required field that was not required; a retired
name in use again; a union or enum without its `unknown` variant; a frozen corpus file changed.
"""
import copy, hashlib, json, sys
from pathlib import Path

SCHEMA = Path('protocol.schema.json')
LOCK = Path('verification/protocol-v1.lock.json')


def sig(s):
    """A stable type signature for one schema node."""
    if s is True or s == {}:
        return 'any'
    if '$ref' in s:
        return s['$ref'].rsplit('/', 1)[-1]
    for key in ('anyOf', 'oneOf'):
        if key in s:
            parts = sorted(sig(x) for x in s[key])
            return '|'.join(parts)
    t = s.get('type')
    if isinstance(t, list):
        inner = dict(s, type=[x for x in t if x != 'null'][0])
        return sig(inner) + '|null'
    if t == 'array':
        if 'prefixItems' in s:
            return '(' + ','.join(sig(x) for x in s['prefixItems']) + ')'
        items = s.get('items', {})
        if isinstance(items, list):
            return '(' + ','.join(sig(x) for x in items) + ')'
        return '[' + sig(items) + ']'
    if t == 'object' and 'additionalProperties' in s and 'properties' not in s:
        return '{' + sig(s['additionalProperties']) + '}'
    if t == 'integer':
        return s.get('format', 'integer')
    return t or 'any'


def fields(node, skip=None):
    props = node.get('properties', {})
    req = set(node.get('required', []))
    return {k: {'type': sig(v), 'required': k in req} for k, v in props.items() if k != skip}


def variant_tag(entry):
    for k, v in entry.get('properties', {}).items():
        if not isinstance(v, dict):
            continue
        if 'const' in v:
            return k, v['const']
        if isinstance(v.get('enum'), list) and len(v['enum']) == 1:
            return k, v['enum'][0]
    return None, None


def flatten(schema):
    out = {}
    for name, d in schema['$defs'].items():
        if 'enum' in d and d.get('type') == 'string':
            out[name] = {'shape': 'enum', 'variants': {v: {} for v in d['enum']}}
        elif 'oneOf' in d or 'anyOf' in d:
            entries = d.get('oneOf') or d.get('anyOf')
            tags, variants = set(), {}
            for e in entries:
                if 'enum' in e and e.get('type') == 'string':
                    for v in e['enum']:
                        variants[v] = {}
                    tags.add('<plain>')
                    continue
                if 'const' in e and e.get('type') == 'string':
                    variants[e['const']] = {}
                    tags.add('<plain>')
                    continue
                tag, value = variant_tag(e)
                if tag is None:
                    variants['<untagged:' + sig(e) + '>'] = {}
                    continue
                tags.add(tag)
                variants[value] = fields(e, skip=tag)
            out[name] = {'shape': 'union', 'tag': '|'.join(sorted(tags)), 'variants': variants}
        elif d.get('type') == 'object' or 'properties' in d:
            out[name] = {'shape': 'struct', 'fields': fields(d)}
        else:
            out[name] = {'shape': 'alias', 'type': sig(d)}
    return out


def compare(lock, cur):
    bad = []
    base = lock['surface']
    retired = set(lock.get('retired', []))
    for name, old in base.items():
        new = cur.get(name)
        if new is None:
            bad.append(f'{name}: definition removed')
            continue
        if new['shape'] != old['shape']:
            bad.append(f'{name}: shape {old["shape"]} -> {new["shape"]}')
            continue
        if old['shape'] == 'alias' and new['type'] != old['type']:
            bad.append(f'{name}: type {old["type"]} -> {new["type"]}')
        if old['shape'] == 'union' and new['tag'] != old['tag']:
            bad.append(f'{name}: tag key {old["tag"]} -> {new["tag"]}')
        if old['shape'] == 'struct':
            bad += field_diff(name, old['fields'], new['fields'])
        if old['shape'] in ('union', 'enum'):
            for v, f in old['variants'].items():
                if v not in new['variants']:
                    bad.append(f'{name}.{v}: variant removed')
                else:
                    bad += field_diff(f'{name}.{v}', f, new['variants'][v])
            for v, f in new['variants'].items():
                if v not in old['variants']:
                    bad += [f'{name}.{v}.{k}: new variant has required field' for k, x in f.items() if x['required']]
    for name, new in cur.items():
        if new['shape'] in ('union', 'enum') and 'unknown' not in new['variants']:
            bad.append(f'{name}: no `unknown` variant, so an older reader cannot tolerate a new one')
        names = [name] + [f'{name}.{v}' for v in new.get('variants', {})] + [f'{name}.{f}' for f in new.get('fields', {})]
        names += [f'{name}.{v}.{f}' for v, fs in new.get('variants', {}).items() for f in fs]
        bad += [f'{n}: retired name reused' for n in names if n in retired]
    return bad


def field_diff(where, old, new):
    bad = []
    for k, f in old.items():
        if k not in new:
            bad.append(f'{where}.{k}: field removed')
        elif new[k]['type'] != f['type']:
            bad.append(f'{where}.{k}: type {f["type"]} -> {new[k]["type"]}')
        elif new[k]['required'] and not f['required']:
            bad.append(f'{where}.{k}: became required')
    for k, f in new.items():
        if k not in old and f['required']:
            bad.append(f'{where}.{k}: new field is required')
    return bad


def corpus_hashes(paths):
    return {p: hashlib.sha256(Path(p).read_bytes()).hexdigest() for p in paths}


def check():
    lock = json.loads(LOCK.read_text())
    bad = compare(lock, flatten(json.loads(SCHEMA.read_text())))
    for p, h in lock.get('corpus', {}).items():
        if not Path(p).is_file():
            bad.append(f'{p}: frozen corpus file missing')
        elif corpus_hashes([p])[p] != h:
            bad.append(f'{p}: frozen corpus file changed (old logs are immutable; add a new file instead)')
    for line in bad:
        print('FAIL', line)
    print(f'ir-compat: {"FAIL" if bad else "PASS"} ({len(lock["surface"])} locked definitions)')
    return 1 if bad else 0


def control():
    schema = json.loads(SCHEMA.read_text())
    lock = json.loads(LOCK.read_text())
    d = schema['$defs']

    def item_variant(s, kind):
        return next(e for e in s['$defs']['Item']['oneOf'] if variant_tag(e)[1] == kind)

    def m_remove_field(s): del s['$defs']['Usage']['properties']['output_tokens']
    def m_rename_variant(s): item_variant(s, 'agent_message')['properties']['kind']['const'] = 'assistant_message'
    def m_type_change(s): s['$defs']['Usage']['properties']['input_tokens'] = {'type': 'string'}
    def m_new_required(s):
        s['$defs']['Usage']['properties']['model'] = {'type': 'string'}
        s['$defs']['Usage']['required'].append('model')
    def m_now_required(s): s['$defs']['Usage']['required'].append('reasoning_tokens')
    def m_remove_enum(s): s['$defs']['ToolKind']['enum'].remove('fetch')
    def m_drop_unknown(s): s['$defs']['ToolKind']['enum'].remove('unknown')
    def m_remove_def(s): del s['$defs']['Location']
    def m_tag_key(s):
        for e in s['$defs']['Item']['oneOf']:
            p = e.get('properties', {})
            if 'kind' in p:
                p['type'] = p.pop('kind')
                e['required'] = ['type' if r == 'kind' else r for r in e.get('required', [])]
    broken = [m_remove_field, m_rename_variant, m_type_change, m_new_required, m_now_required,
              m_remove_enum, m_drop_unknown, m_remove_def, m_tag_key]

    def a_optional_field(s): s['$defs']['Usage']['properties']['model'] = {'type': ['string', 'null']}
    def a_variant(s): s['$defs']['ToolKind']['enum'].append('browse')
    def a_definition(s): s['$defs']['Citation'] = {'type': 'object', 'properties': {'url': {'type': 'string'}}, 'required': ['url']}
    allowed = [a_optional_field, a_variant, a_definition]

    failed = 0
    for m in broken:
        s = copy.deepcopy(schema); m(s)
        if not compare(lock, flatten(s)):
            print('CONTROL FAIL: accepted', m.__name__); failed += 1
    for a in allowed:
        s = copy.deepcopy(schema); a(s)
        got = compare(lock, flatten(s))
        if got:
            print('CONTROL FAIL: refused', a.__name__, got); failed += 1
    print(f'ir-compat control: {"FAIL" if failed else "PASS"} ({len(broken)} refused, {len(allowed)} accepted)')
    return 1 if failed else 0


def lock():
    old = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    corpus = sorted(old.get('corpus', {})) or sorted(str(p) for p in Path('tests/fixtures/ir-v1').glob('*.jsonl'))
    out = {
        'ir': 'bboxy log IR',
        'version': {'major': 1, 'minor': 0},
        'spec': '.bboxy/specs/log-ir-v1.md',
        'retired': old.get('retired', []),
        'corpus': corpus_hashes(corpus),
        'surface': flatten(json.loads(SCHEMA.read_text())),
    }
    LOCK.write_text(json.dumps(out, indent=1, sort_keys=True) + '\n')
    print(f'locked {len(out["surface"])} definitions, {len(corpus)} corpus files')
    return 0


if __name__ == '__main__':
    sys.exit({'check': check, 'control': control, 'lock': lock}[sys.argv[1]]())
