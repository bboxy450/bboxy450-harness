# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Causal production controls for omitted instructions and empty MCP forwarding."""
import json
import hashlib
from pathlib import Path
import subprocess
import sys
import os
import tempfile
sys.dont_write_bytecode = True
ROOT = Path(__file__).resolve().parents[1]
SOURCE = ROOT / 'src/backend/acp_agent.rs'
COMMAND = ['cargo','test','--offline','--locked','--test','acp_launch_context','downstream_context_and_callable_mcp','--','--exact','--nocapture']
original = SOURCE.read_bytes()
logs = Path(tempfile.mkdtemp(prefix='harness-launch-controls-',dir='/tmp'))
rows=[]
try:
    for name, guard, broken, assertion in [
        ('prompt', 'if !self.launch_context.is_empty() {', 'if false {', 'missing admitted launcher context'),
        ('mcp', '"mcpServers": self.mcp_servers', '"mcpServers": []', 'missing callable admitted MCP server'),
    ]:
        text=original.decode()
        if text.count(guard)!=1:
            raise RuntimeError('launch control guard changed: '+name)
        SOURCE.write_text(text.replace(guard,broken,1))
        red=subprocess.run(COMMAND,cwd=ROOT,env=os.environ.copy(),capture_output=True,text=True,timeout=180)
        log=logs/(name+'-red.log');log.write_text(red.stdout+red.stderr)
        if red.returncode!=101 or assertion not in red.stdout+red.stderr:
            raise RuntimeError('launch control did not fail named assertion: '+name+' '+(red.stdout+red.stderr)[-2000:])
        rows.append({'control':name,'red_exit':red.returncode,'assertion':assertion,'sha256':hashlib.sha256(log.read_bytes()).hexdigest()})
finally:
    SOURCE.write_bytes(original)
green=subprocess.run(COMMAND,cwd=ROOT,env=os.environ.copy(),capture_output=True,text=True,timeout=180)
log=logs/'restored-green.log';log.write_text(green.stdout+green.stderr)
if green.returncode or SOURCE.read_bytes()!=original:
    raise RuntimeError('launch controls did not restore green source: '+(green.stdout+green.stderr)[-2000:])
print(json.dumps({'controls':rows,'command':COMMAND,'green_exit':green.returncode,'source_sha256':hashlib.sha256(original).hexdigest(),'green_sha256':hashlib.sha256(log.read_bytes()).hexdigest(),'logs':str(logs),'operator_input':False}))
