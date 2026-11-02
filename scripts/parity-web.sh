#!/bin/sh
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
# Web vs native parity for the rich face: one replay JSONL rendered by the web reference
# (bboxy450-term lab/web/harness.js, headless Chrome) and by the native GUI (--capture), same size,
# dark theme. Writes web.png, native.png, side.png (web | native | diff) and score.json into OUT.
#   sh scripts/parity-web.sh REPLAY.jsonl OUT [WxH]    LAB=path/to/bboxy450-term/lab/web BIN=path/to/bboxy450-harness
# MENU=profiles|slash renders that floating menu open on both sides; the profile list is the
# fixture home PROFILES_HOME (default tests/fixtures/gui/profiles-home), never the operator's.
# The fixture's env-key profiles name BBOXY450_PARITY_KEY (set here, a placeholder, never a key)
# and BBOXY450_PARITY_UNSET_KEY (never set), so `env` and `missing env` render on both sides.
set -eu
REPLAY=$(cd "$(dirname "$1")" && pwd)/$(basename "$1"); OUT=$2; SIZE=${3:-760x1400}
W=${SIZE%x*}; H=${SIZE#*x}
ROOT=$(cd "$(dirname "$0")/.." && pwd)
LAB=${LAB:-$ROOT/../bboxy450-term/lab/web}; LAB=$(cd "$LAB" && pwd)
BIN=${BIN:-$ROOT/target/debug/bboxy450-harness}
MENU=${MENU:-}; PROFILES_HOME=${PROFILES_HOME:-$ROOT/tests/fixtures/gui/profiles-home}
CHROME=${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}
mkdir -p "$OUT"; OUT=$(cd "$OUT" && pwd)
if [ -n "$MENU" ]; then
  export BBOXY450_PARITY_KEY=placeholder; unset BBOXY450_PARITY_UNSET_KEY 2>/dev/null || true
  HOME="$PROFILES_HOME" "$BIN" gui --replay "$REPLAY" --capture "${W}x${H}" --open-menu "$MENU" --screenshot "$OUT/native.png"
else
  "$BIN" gui --replay "$REPLAY" --capture "${W}x${H}" --screenshot "$OUT/native.png"
fi
python3 -I - "$REPLAY" "$OUT/page.html" "$LAB" "$W" "$H" "$MENU" "$PROFILES_HOME/.bboxy450-harness/profiles" <<'PY'
import json, os, sys
replay, page, lab, w, h, menu, pdir = sys.argv[1:]
envs = [json.loads(l) for l in open(replay) if l.strip()]
# the web face's profile records, from the same fixture files the native capture lists
profiles = []
if menu and os.path.isdir(pdir):
    default = open(os.path.join(pdir, '..', 'default-profile')).read().strip() if os.path.isfile(os.path.join(pdir, '..', 'default-profile')) else ''
    for f in sorted(os.listdir(pdir)):
        if not f.endswith('.md'): continue
        kv, command, env_names, block = {}, [], [], None
        for l in open(os.path.join(pdir, f)).read().split('---')[1].splitlines():
            if ':' not in l: continue
            k, v = (x.strip() for x in l.split(':', 1))
            depth = len(l) - len(l.lstrip())
            if depth == 0: kv[k] = v.strip('"'); block = k
            elif block == 'agent' and depth == 2 and k == 'command': command = [x.strip() for x in v.strip('[]').split(',') if x.strip()]
            elif block == 'agent' and depth == 2 and k == 'env': block = 'env'
            elif block == 'env' and depth >= 4: env_names.append(k)
        key = kv.get('api_key', '')
        auth = ({'kind': 'env', 'ref': kv['api_key_env'], 'ok': kv['api_key_env'] in os.environ} if 'api_key_env' in kv
                else {'kind': 'keychain', 'ok': True} if key.startswith('!security ') else {'kind': 'command', 'ok': True} if key.startswith('!')
                else {'kind': 'none', 'ok': True})
        p = {'name': f[:-3], 'provider': kv.get('provider', ''), 'model': kv.get('model', ''), 'endpoint': kv.get('endpoint', ''), 'auth': auth, 'default': f[:-3] == default}
        if 'context_tokens' in kv: p['context_tokens'] = int(kv['context_tokens'])
        if kv.get('provider') == 'acp-agent': p.update(kind='acp-agent', agent_command=command, agent_env_names=env_names)
        profiles.append(p)
opts = json.dumps({'interactive': True, 'profiles': profiles}) if menu else '{}'
open_menu = {'profiles': 'v.openProfiles();', 'slash': "v.compose({ focus: true, text: '/', menu: 'slash', sel: 0 });"}.get(menu, '')
src = lambda f: 'file://' + lab + '/' + f
open(page, 'w').write(f'''<!doctype html><html><head><meta charset="utf-8">
<link rel="stylesheet" href="{src('lab.css')}">
<script src="{src('tokens.js')}"></script><script src="{src('lab.js')}"></script><script src="{src('components.js')}"></script><script src="{src('harness.js')}"></script>
<style>html,body{{margin:0;height:100%;overflow:hidden;background:var(--bg)}} #f{{width:{w}px;height:{h}px;--seat:var(--accent)}}</style>
</head><body><div id="f"></div><script>
Lab.prefs.theme = 'dark'; Lab.prefs.density = 'cozy'; Lab.applyTokens();
document.documentElement.classList.add('parity');
const v = Hx.view({opts}); document.getElementById('f').append(v.node);
{json.dumps(envs)}.forEach(e => v.apply(e));
{open_menu}
</script></body></html>''')
PY
"$CHROME" --headless=new --use-mock-keychain --password-store=basic --disable-gpu --hide-scrollbars --force-device-scale-factor=1 --user-data-dir="$OUT/.chrome" \
  --window-size="$W,$H" --virtual-time-budget=2000 --screenshot="$OUT/web.png" "file://$OUT/page.html" >/dev/null 2>&1
rm -rf "$OUT/.chrome"
python3 -I - "$OUT" <<'PY'
import json, sys
from PIL import Image, ImageChops, ImageStat
out = sys.argv[1]
web = Image.open(out + '/web.png').convert('RGB'); nat = Image.open(out + '/native.png').convert('RGB')
nat = nat.resize(web.size) if nat.size != web.size else nat
diff = ImageChops.difference(web, nat)
side = Image.new('RGB', (web.width * 3 + 16, web.height), (40, 40, 48))
for i, im in enumerate((web, nat, diff)): side.paste(im, (i * (web.width + 8), 0))
side.save(out + '/side.png')
mean = sum(ImageStat.Stat(diff.convert('L')).mean) / 255
# row profile: share of rows whose ink presence differs (structure, not colour)
def ink(im): g = im.convert('L'); px = g.load(); return [sum(1 for x in range(0, g.width, 4) if abs(px[x, y] - px[0, y]) > 24) > 2 for y in range(g.height)]
a, b = ink(web), ink(nat)
score = {'mean_abs_diff': round(mean, 4), 'row_structure_mismatch': round(sum(x != y for x, y in zip(a, b)) / len(a), 4), 'size': list(web.size)}
json.dump(score, open(out + '/score.json', 'w')); print('parity', json.dumps(score))
PY
