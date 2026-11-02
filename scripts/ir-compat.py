# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Bboxy log IR evolution gate (.bboxy/specs/log-ir-v1.md §2): only add, never change or remove.

  python3 -I scripts/ir-compat.py check      protocol.schema.json against verification/protocol-v1.lock.json
  python3 -I scripts/ir-compat.py control    each forbidden mutation of the current schema must be refused
  python3 -I scripts/ir-compat.py lock       refresh the lock (`make lock-ir`); refused unless additive over the old lock

The surface is flattened from the schemars export: every definition, every field with its type
signature and required flag, every variant of a tagged union or string enum. Allowed: a new
definition, a new optional field, a new variant. Refused: a removed definition, field or variant;
a changed type, tag key or definition shape; a required field that was not required; a retired
name in use again; a union or enum without a catch-all (a named `unknown` value or an untagged\n`true` entry: the two are the same variant to the gate); a frozen corpus file changed.
"""
import argparse, copy, hashlib, json, sys
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


def is_catch_all(e):
    """An untagged `Unknown(Value)` variant exports as `true` (or `{}`): it accepts anything."""
    return e is True or e == {}


def is_string_enum(e):
    return isinstance(e, dict) and e.get('type') == 'string' and ('enum' in e or 'const' in e)


def union_entries(d):
    """The variant entries of a union, nested anyOf/oneOf flattened; returns (entries, has_catch_all)."""
    out, catch = [], False
    for e in d.get('oneOf') or d.get('anyOf') or []:
        if is_catch_all(e):
            catch = True
        elif isinstance(e, dict) and ('oneOf' in e or 'anyOf' in e) and 'properties' not in e:
            inner, c = union_entries(e)
            out += inner; catch = catch or c
        else:
            out.append(e)
    return out, catch


def variant_tag(entry):
    if not isinstance(entry, dict):
        return None, None
    for k, v in entry.get('properties', {}).items():
        if not isinstance(v, dict):
            continue
        if 'const' in v:
            return k, v['const']
        if isinstance(v.get('enum'), list) and len(v['enum']) == 1:
            return k, v['enum'][0]
    return None, None


def flatten(schema):
    """One shape per definition. The reader-side catch-all (a named `unknown` value or an untagged
    `true` entry) always appears as the variant `unknown`, so switching between the two forms is
    not a change: both accept every value the other does, and writers never emit either (E5)."""
    out = {}
    for name, d in schema['$defs'].items():
        if is_string_enum(d):
            out[name] = {'shape': 'enum', 'variants': {v: {} for v in d.get('enum', [d.get('const')])}}
        elif 'oneOf' in d or 'anyOf' in d:
            entries, catch = union_entries(d)
            if len(entries) == 1 and is_string_enum(entries[0]):
                e = entries[0]
                variants = {v: {} for v in e.get('enum', [e.get('const')])}
                if catch:
                    variants['unknown'] = {}
                out[name] = {'shape': 'enum', 'variants': variants}
                continue
            tags, variants = set(), {}
            for e in entries:
                if is_string_enum(e):
                    for v in e.get('enum', [e.get('const')]):
                        variants[v] = {}
                    tags.add('<plain>')
                    continue
                tag, value = variant_tag(e)
                if tag is None:
                    variants['<untagged:' + sig(e) + '>'] = {}
                    continue
                tags.add(tag)
                variants[value] = fields(e, skip=tag)
            if catch:
                variants['unknown'] = {}
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
            bad.append(f'{name}: no `unknown` value or untagged catch-all, so an older reader cannot tolerate a new variant')
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


def check(schema_path=SCHEMA, fragment=None, require_current=False):
    lock = json.loads(LOCK.read_text())
    schema = json.loads(Path(schema_path).read_text())
    if fragment:
        # Apply to a copy, never the canonical schema or the frozen lock.
        schema['$defs'].update(json.loads(Path(fragment).read_text())['$defs'])
    bad = compare(lock, flatten(schema))
    if require_current and not bad and lock['surface'] != flatten(schema):
        bad.append('schema has unlocked additions; run make lock-ir')
    for p, h in lock.get('corpus', {}).items():
        if not Path(p).is_file():
            bad.append(f'{p}: frozen corpus file missing')
        elif corpus_hashes([p])[p] != h:
            bad.append(f'{p}: frozen corpus file changed (old logs are immutable; add a new file instead)')
    for line in bad:
        print('FAIL', line)
    print(f'ir-compat: {"FAIL" if bad else "PASS"} ({len(lock["surface"])} locked definitions)')
    return 1 if bad else 0


def enum_node(s, name):
    """The node holding a string enum's values, whether bare or wrapped in anyOf[enum, true]."""
    d = s['$defs'][name]
    if is_string_enum(d):
        return d
    return next(e for e in (d.get('oneOf') or d.get('anyOf')) if is_string_enum(e))


def variants_of(s, name):
    d = s['$defs'][name]
    return [e for e in (d.get('oneOf') or d.get('anyOf')) if isinstance(e, dict)]


def drop_catch_all(s, name):
    d = s['$defs'][name]
    if is_string_enum(d):
        d['enum'].remove('unknown')
        return
    key = 'oneOf' if 'oneOf' in d else 'anyOf'
    d[key] = [e for e in d[key] if not is_catch_all(e)]
    for e in d[key]:
        if is_string_enum(e) and 'unknown' in e.get('enum', []):
            e['enum'].remove('unknown')
    for e in d[key]:
        if variant_tag(e)[1] == 'unknown':
            d[key].remove(e); break


def untagged_form(schema):
    """The current schema as schemars exports it once every `Unknown` is an untagged `Value`:
    the named `unknown` value leaves each enum and a bare `true` entry joins the union."""
    s = copy.deepcopy(schema)
    for name, d in s['$defs'].items():
        if is_string_enum(d) and 'unknown' in d.get('enum', []):
            d['enum'].remove('unknown')
            s['$defs'][name] = {'anyOf': [d, True]}
        elif 'oneOf' in d or 'anyOf' in d:
            key = 'oneOf' if 'oneOf' in d else 'anyOf'
            kept = [e for e in d[key] if not (isinstance(e, dict) and variant_tag(e)[1] == 'unknown')]
            if len(kept) != len(d[key]):
                d[key] = kept + [True]
    return s


SYN_DEF, SYN_FIELD, SYN_VARIANT = '__IrCompatControlAdded', '__ir_compat_control_field', '__ir_compat_control_variant'


def control():
    schema = json.loads(SCHEMA.read_text())
    lock = json.loads(LOCK.read_text())
    # The synthetic additions must never collide with a real one (Citation and Usage.model did):
    # reserved names, and a loud failure if one ever appears in the schema or the lock.
    present = set(schema['$defs']) | set(lock['surface']) | set(schema['$defs']['Usage'].get('properties', {})) | set(enum_node(schema, 'ToolKind')['enum'])
    clash = {SYN_DEF, SYN_FIELD, SYN_VARIANT} & present
    if clash:
        print('CONTROL FAIL: reserved synthetic name present in the schema or lock:', sorted(clash)); return 1

    def item_variant(s, kind):
        return next(e for e in variants_of(s, 'Item') if variant_tag(e)[1] == kind)

    def m_remove_field(s): del s['$defs']['Usage']['properties']['output_tokens']
    def m_rename_variant(s): item_variant(s, 'agent_message')['properties']['kind']['const'] = 'assistant_message'
    def m_type_change(s): s['$defs']['Usage']['properties']['input_tokens'] = {'type': 'string'}
    def m_new_required(s):
        s['$defs']['Usage']['properties'][SYN_FIELD] = {'type': 'string'}
        s['$defs']['Usage']['required'].append(SYN_FIELD)
    def m_now_required(s): s['$defs']['Usage']['required'].append('reasoning_tokens')
    def m_remove_enum(s): enum_node(s, 'ToolKind')['enum'].remove('fetch')
    def m_drop_unknown(s): drop_catch_all(s, 'ToolKind')
    def m_drop_item_unknown(s): drop_catch_all(s, 'Item')
    def m_remove_def(s): del s['$defs']['Location']
    def m_tag_key(s):
        for e in variants_of(s, 'Item'):
            p = e.get('properties', {})
            if 'kind' in p:
                p['type'] = p.pop('kind')
                e['required'] = ['type' if r == 'kind' else r for r in e.get('required', [])]
    broken = [m_remove_field, m_rename_variant, m_type_change, m_new_required, m_now_required,
              m_remove_enum, m_drop_unknown, m_drop_item_unknown, m_remove_def, m_tag_key]

    def a_optional_field(s): s['$defs']['Usage']['properties'][SYN_FIELD] = {'type': ['string', 'null']}
    def a_variant(s): enum_node(s, 'ToolKind')['enum'].append(SYN_VARIANT)
    def a_definition(s): s['$defs'][SYN_DEF] = {'type': 'object', 'properties': {'url': {'type': 'string'}}, 'required': ['url']}
    def a_untagged_unknown(s): s['$defs'].update(untagged_form(s)['$defs'])
    allowed = [a_optional_field, a_variant, a_definition, a_untagged_unknown]

    failed = 0
    for label, base in (('current', schema), ('untagged', untagged_form(schema))):
        if compare(lock, flatten(base)):
            print('CONTROL FAIL: the', label, 'form of the schema is refused unmodified', compare(lock, flatten(base))); failed += 1
        for m in broken:
            s = copy.deepcopy(base); m(s)
            if not compare(lock, flatten(s)):
                print('CONTROL FAIL: accepted', m.__name__, 'on the', label, 'form'); failed += 1
        for a in allowed:
            s = copy.deepcopy(base); a(s)
            got = compare(lock, flatten(s))
            if got:
                print('CONTROL FAIL: refused', a.__name__, 'on the', label, 'form', got); failed += 1
    print(f'ir-compat control: {"FAIL" if failed else "PASS"} ({len(broken)} refused, {len(allowed)} accepted, both schema forms)')
    return 1 if failed else 0


def lock():
    """Refresh the lock. Refused unless the current schema is an addition over the existing lock
    (so a re-lock can never enshrine a removal or rename) and every frozen corpus file is intact."""
    old = json.loads(LOCK.read_text()) if LOCK.exists() else {}
    surface = flatten(json.loads(SCHEMA.read_text()))
    if old:
        bad = compare(old, surface)
        for p, h in old.get('corpus', {}).items():
            if not Path(p).is_file() or corpus_hashes([p])[p] != h:
                bad.append(f'{p}: frozen corpus file missing or changed')
        if bad:
            for line in bad: print('FAIL', line)
            print('ir-compat lock: REFUSED, the change is not additive over the existing lock')
            return 1
    corpus = sorted(set(old.get('corpus', {})) | {str(p) for p in Path('tests/fixtures/ir-v1').glob('*.jsonl')})
    out = {
        'ir': 'bboxy log IR',
        'version': {'major': 1, 'minor': 0},
        'spec': '.bboxy/specs/log-ir-v1.md',
        'retired': old.get('retired', []),
        'corpus': corpus_hashes(corpus),
        'surface': surface,
    }
    LOCK.write_text(json.dumps(out, indent=1, sort_keys=True) + '\n')
    print(f'locked {len(out["surface"])} definitions, {len(corpus)} corpus files')
    return 0


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('command', choices=['check', 'control', 'lock'])
    parser.add_argument('--schema', type=Path, default=SCHEMA)
    parser.add_argument('--fragment', type=Path)
    parser.add_argument('--require-current', action='store_true')
    args = parser.parse_args()
    sys.exit(check(args.schema, args.fragment, args.require_current) if args.command == 'check' else {'control': control, 'lock': lock}[args.command]())
