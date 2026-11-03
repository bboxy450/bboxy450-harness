# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Private downstream ACP/exec compatibility process; never invokes a provider."""
import json
import os
from pathlib import Path
import subprocess
import sys


def emit(value):
    print(json.dumps(value), flush=True)


def server():
    # A throwaway file only: no capability bytes are returned or journaled.
    capability = os.environ.get('BBOXY_CAPABILITY_FILE')
    custody = bool(capability and Path(capability).stat().st_uid == os.getuid()
                   and Path(capability).stat().st_mode & 0o777 == 0o600
                   and __import__('hashlib').sha256(Path(capability).read_bytes()).hexdigest() == os.environ.get('PARITY_CAP_HASH'))
    for line in sys.stdin:
        request = json.loads(line)
        method = request.get('method')
        result = None
        if method == 'initialize':
            if os.environ.get('PARITY_SLOW'):
                import time
                time.sleep(60)
            result = {'protocolVersion':'2025-11-25','capabilities':{'tools':{}},'serverInfo':{'name':'private-parity','version':'1'}}
        elif method == 'tools/list':
            result = {'tools':[{'name':'identity','inputSchema':{'type':'object'}}]}
        elif method == 'tools/call':
            if os.environ.get('PARITY_SLOW_CALL'):
                import time
                time.sleep(60)
            result = {'content':[{'type':'text','text':json.dumps({'seat':os.environ.get('BBOXY_SEAT_ID'), 'custody':custody, 'launcher':os.environ.get('PARITY_ORIGIN') == 'fixture-origin:launcher', 'probe':Path(capability).read_text().strip() if custody and os.environ.get('PARITY_REFLECT') else None})}]}
            with open(os.environ['PARITY_EFFECTS'], 'a') as out:
                out.write('tool-effect\n')
        if 'id' in request:
            emit({'jsonrpc':'2.0','id':request['id'],'result':result or {}})


def call_server(config):
    command = [config['command'], *config.get('args', [])]
    # ACP stdio env is empty: capability custody stays on the host side of relay.
    assert config['env'] == []
    child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
    try:
        def rpc(i, method, params):
            child.stdin.write(json.dumps({'jsonrpc':'2.0','id':i,'method':method,'params':params})+'\n')
            child.stdin.flush()
            line = child.stdout.readline()
            return json.loads(line)['result'] if line else None
        if rpc(1, 'initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'private','version':'1'}}) is None:
            return None
        rpc(2, 'tools/list', {})
        result = rpc(3, 'tools/call', {'name':'identity','arguments':{}})
        return json.loads(result['content'][0]['text']) if result else None
    except (BrokenPipeError, ValueError, KeyError):
        return None
    finally:
        try:
            child.stdin.close()
        except OSError:
            pass
        try:
            if child.poll() is None:
                child.terminate()
        except ProcessLookupError:
            pass
        child.wait(timeout=3)


def agent():
    journal = Path(sys.argv[2])
    servers = []
    # argv and wire evidence contains no environment values or capability bytes.
    with journal.open('a') as out:
        out.write(json.dumps({'argv':sys.argv[3:],'cwd':os.getcwd()})+'\n')
    for line in sys.stdin:
        request = json.loads(line)
        with journal.open('a') as out:
            out.write(json.dumps(request)+'\n')
        method = request.get('method')
        result = {}
        if method == 'initialize':
            if '--hang-startup' in sys.argv:
                continue
            result = {'protocolVersion':1,'agentInfo':{'name':sys.argv[3],'version':'fixture'}}
        elif method == 'session/new':
            servers = request['params']['mcpServers']
            result = {'sessionId':'private-acp','configOptions':[{'id':'model','category':'model','currentValue':'default','options':[{'value':'fixture/model-A','name':'A'},{'value':'fixture/model-B','name':'B'}]}]}
        elif method == 'session/prompt':
            if '--crash-first' in sys.argv and sum('"argv"' in row for row in journal.read_text().splitlines()) == 1:
                return
            emit({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{
                'sessionId':'private-acp','toolCall':{'toolCallId':__import__('uuid').uuid4().hex,'title':'Permission semantics probe','kind':'execute'},
                'options':[{'optionId':'yes','name':'Allow','kind':'allow_once'}, {'optionId':'no','name':'Deny','kind':'reject_once'}]}})
            permission = json.loads(sys.stdin.readline())
            with journal.open('a') as out:
                out.write(json.dumps(permission)+'\n')
            permission = permission['result']['outcome'].get('optionId')
            tools = {s['name']:call_server(s) for s in servers}
            emit({'jsonrpc':'2.0','method':'session/update','params':{'sessionId':'private-acp','update':{'sessionUpdate':'agent_message_chunk','content':{'type':'text','text':json.dumps({'tools':tools,'permission':permission})}}}})
            result = {'stopReason':'end_turn'}
        elif method == 'session/cancel':
            continue
        if 'id' in request:
            emit({'jsonrpc':'2.0','id':request['id'],'result':result})

if sys.argv[1] == 'server':
    server()
else:
    agent()
