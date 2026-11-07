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
import threading
import time

FRAME_LIMIT = 64 * 1024
JOURNAL_LIMIT = 256 * 1024
STDERR_LIMIT = 16 * 1024


def redacted(text):
    # Only the throwaway capability created by isolated(); never resolve the
    # operator's capability path. Redact before retaining any reflected reply.
    if not os.environ.get('PARITY_CHILD'):
        return text
    path = Path(os.environ['HOME']) / 'capability'
    if path.is_file() and path.stat().st_uid == os.getuid() and path.stat().st_mode & 0o777 == 0o600:
        with path.open('r') as source:
            secret = source.read(4096).strip()
        if secret:
            text = text.replace(secret, '<redacted>')
            for size in range(len(secret) - 1, 0, -1):
                if text.endswith(secret[:size]):
                    text = text[:-size] + '<redacted>'
                    break
    return text


def bounded_text(text, limit):
    # ASCII escaped prefixes keep the serialized evidence bound independent of
    # arbitrary UTF-8, while byte counts/truncation remain explicit.
    escaped = redacted(text).encode('ascii', errors='backslashreplace').decode()
    return escaped[:limit], len(escaped) > limit


def journal_record(path, value):
    line = redacted(json.dumps(value, ensure_ascii=True)) + '\n'
    raw = line.encode()
    if len(raw) > FRAME_LIMIT:
        raise ValueError('private journal record exceeds64KiB')
    fd = os.open(path, os.O_WRONLY | os.O_APPEND | os.O_CREAT | os.O_NOFOLLOW, 0o600)
    with os.fdopen(fd, 'ab') as out:
        if os.fstat(out.fileno()).st_size + len(raw) > JOURNAL_LIMIT:
            raise ValueError('private journal exceeds256KiB')
        out.write(raw)


def bounded_lines():
    while True:
        line = sys.stdin.buffer.readline(FRAME_LIMIT + 1)
        if not line:
            return
        if len(line) > FRAME_LIMIT or not line.endswith(b'\n'):
            raise ValueError('private RPC input exceeds64KiB or lacks newline')
        yield line


class RpcFailure(Exception):
    pass


def emit(value):
    print(json.dumps(value), flush=True)


def server():
    # A throwaway file only: no capability bytes are returned or journaled.
    capability = os.environ.get('BBOXY_CAPABILITY_FILE')
    custody = bool(capability and Path(capability).stat().st_uid == os.getuid()
                   and Path(capability).stat().st_mode & 0o777 == 0o600
                   and __import__('hashlib').sha256(Path(capability).read_bytes()).hexdigest() == os.environ.get('PARITY_CAP_HASH'))
    for line in bounded_lines():
        request = json.loads(line)
        method = request.get('method')
        result = None
        if method == 'initialize':
            if os.environ.get('PARITY_WAIT_INITIALIZE'):
                while not Path(os.environ['PARITY_WAIT_INITIALIZE']).exists():
                    time.sleep(0.005)
            if os.environ.get('PARITY_SLOW'):
                time.sleep(60)
            result = {'protocolVersion':'2025-11-25','capabilities':{'tools':{}},'serverInfo':{'name':'bboxy450' if custody else 'private-parity','version':'1'}}
        elif method == 'tools/list':
            names = ['identity', 'python_execution_context'] if custody else ['identity']
            result = {'tools':[{'name':name,'inputSchema':{'type':'object'}} for name in names]}
        elif method == 'tools/call':
            if request['params']['name'] == 'python_execution_context':
                context = {'guard_version':1, 'seat_id':os.environ.get('BBOXY_SEAT_ID'),
                           'process_generation':1, 'grant_epoch':1, 'provider_revision':1,
                           'knowledge_binding':{'root_id':'private-fixture','revision':1},
                           'workspace':None,'cwd':None,'target':{'kind':'local'}}
                emit({'jsonrpc':'2.0','id':request['id'],'result':{
                    'content':[{'type':'text','text':json.dumps({'context':context})}],
                    'structuredContent':{'context':context},'_meta':{'bboxy450/guard_transport_version':1}}})
                continue
            if os.environ.get('PARITY_SLOW_CALL'):
                time.sleep(60)
            result = {'content':[{'type':'text','text':json.dumps({'seat':os.environ.get('BBOXY_SEAT_ID'), 'custody':custody, 'launcher':os.environ.get('PARITY_ORIGIN') == 'fixture-origin:launcher', 'probe':Path(capability).read_text().strip() if custody and os.environ.get('PARITY_REFLECT') else None})}]}
            with open(os.environ['PARITY_EFFECTS'], 'a') as out:
                out.write('tool-effect\n')
        if 'id' in request:
            emit({'jsonrpc':'2.0','id':request['id'],'result':result or {}})


def call_server(config, journal):
    command = [config['command'], *config.get('args', [])]
    # ACP stdio env is empty: capability custody stays on the host side of relay.
    assert config['env'] == []
    record = {'server': config['name'], 'pid': None, 'stage': 'spawn',
              'responses': [], 'responses_discarded': 0, 'python_execute_requests': 0,
              'error': None, 'settlement': {}}
    try:
        child = subprocess.Popen(command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    except OSError as error:
        record['error'] = {'type': type(error).__name__, 'errno': error.errno}
        journal_record(journal.with_name(journal.name + '.mcp.jsonl'), record)
        return None
    record['pid'] = child.pid
    stderr = {'bytes': bytearray(), 'discarded': 0, 'eof': False, 'error': None}

    def drain():
        try:
            while True:
                chunk = child.stderr.read(4096)
                if not chunk:
                    stderr['eof'] = True
                    return
                keep = min(len(chunk), STDERR_LIMIT - len(stderr['bytes']))
                stderr['bytes'].extend(chunk[:keep])
                stderr['discarded'] += len(chunk) - keep
        except OSError as error:
            stderr['error'] = error.errno

    reader = threading.Thread(target=drain, daemon=True)
    reader.start()
    try:
        def rpc(i, method, params):
            record['stage'] = params.get('name', method)
            if record['stage'] == 'python_execute':
                record['python_execute_requests'] += 1
            child.stdin.write((json.dumps({'jsonrpc':'2.0','id':i,'method':method,'params':params})+'\n').encode())
            child.stdin.flush()
            line = child.stdout.readline(FRAME_LIMIT + 1)
            raw, truncated = bounded_text(line.decode('utf-8', errors='replace'), 1024)
            record['responses'].append({'id': i, 'stage': record['stage'], 'bytes': len(line),
                                        'response': raw, 'truncated': truncated})
            record['responses_discarded'] += max(0, len(record['responses']) - 16)
            del record['responses'][:-16]
            if not line:
                raise RpcFailure('RPC EOF')
            if len(line) > FRAME_LIMIT or not line.endswith(b'\n'):
                raise RpcFailure('RPC response exceeds64KiB or lacks newline')
            response = json.loads(line)
            if response.get('jsonrpc') != '2.0' or response.get('id') != i:
                raise RpcFailure('RPC identity mismatch')
            if 'error' in response:
                raise RpcFailure(json.dumps(response['error']))
            result = response['result']
            if isinstance(result, dict) and result.get('isError'):
                raise RpcFailure(json.dumps(result))
            return result
        rpc(1, 'initialize', {'protocolVersion':'2025-11-25','capabilities':{},'clientInfo':{'name':'private','version':'1'}})
        rpc(2, 'tools/list', {})
        if config['name'] == 'bboxy_python':
            orient = rpc(3, 'tools/call', {'name':'python_execute_orient','arguments':{}})
            if not orient['structuredContent']['ready']:
                raise RpcFailure('Python orientation ready=false')
            started = rpc(4, 'tools/call', {'name':'python_execute','arguments':{
                'source':"import pathlib\nemit({'answer':6*7,'cwd':str(pathlib.Path.cwd())})"}})
            execution = started['structuredContent']['execution_id']
            for i in range(500):
                current = rpc(5+i, 'tools/call', {'name':'python_execution_get','arguments':{'execution_id':execution}})
                current = current['structuredContent']
                if current.get('status') not in ('starting','running','cancelling'):
                    return current
                time.sleep(0.01)
            raise RuntimeError('private Python execution did not settle')
        result = rpc(3, 'tools/call', {'name':'identity','arguments':{}})
        return json.loads(result['content'][0]['text']) if result else None
    except (BrokenPipeError, ValueError, KeyError, TypeError, RpcFailure) as error:
        message, truncated = bounded_text(str(error), 2048)
        record['error'] = {'type': type(error).__name__, 'message': message, 'truncated': truncated}
        return None
    finally:
        until = time.monotonic() + 3
        settlement = record['settlement']
        settlement['prior_status'] = child.poll()
        settlement['term_attempted'] = False
        try:
            child.stdin.close()
        except OSError as error:
            settlement['stdin_close_errno'] = error.errno
        settlement['stdin_closed'] = True
        try:
            if child.poll() is None:
                settlement['term_attempted'] = True
                child.terminate()
        except OSError as error:
            settlement['term_errno'] = error.errno
        try:
            settlement['actual_status'] = child.wait(timeout=max(0, until - time.monotonic()))
        except subprocess.TimeoutExpired:
            settlement['wait_timeout'] = True
            raise
        finally:
            reader.join(timeout=max(0, until - time.monotonic()))
            text, truncated = bounded_text(bytes(stderr['bytes']).decode('utf-8', errors='replace'), 4096)
            record['stderr'] = {'text': text, 'truncated': truncated, 'captured_bytes': len(stderr['bytes']),
                                'discarded_bytes': stderr['discarded'], 'eof': stderr['eof'],
                                'errno': stderr['error'], 'reader_joined': not reader.is_alive()}
            journal_record(journal.with_name(journal.name + '.mcp.jsonl'), record)


def agent():
    journal = Path(sys.argv[2])
    servers = []
    # argv and wire evidence contains no environment values or capability bytes.
    journal_record(journal, {'argv':sys.argv[3:],'cwd':os.getcwd()})
    for line in bounded_lines():
        request = json.loads(line)
        journal_record(journal, request)
        method = request.get('method')
        result = {}
        if method == 'initialize':
            if '--hang-startup' in sys.argv:
                continue
            result = {'protocolVersion':1,'agentInfo':{'name':sys.argv[3],'version':'fixture'}}
            if sys.argv[3].endswith('-sysprompt'):
                result['agentCapabilities'] = {'sessionCapabilities':{'systemPrompt':{}}}
        elif method == 'session/new':
            servers = request['params']['mcpServers']
            result = {'sessionId':'private-acp','configOptions':[{'id':'model','category':'model','currentValue':'default','options':[{'value':'fixture/model-A','name':'A'},{'value':'fixture/model-B','name':'B'}]}]}
        elif method == 'session/prompt':
            if '--crash-first' in sys.argv and sum('"argv"' in row for row in journal.read_text().splitlines()) == 1:
                return
            emit({'jsonrpc':'2.0','id':'permission','method':'session/request_permission','params':{
                'sessionId':'private-acp','toolCall':{'toolCallId':__import__('uuid').uuid4().hex,'title':'Permission semantics probe','kind':'execute'},
                'options':[{'optionId':'yes','name':'Allow','kind':'allow_once'}, {'optionId':'no','name':'Deny','kind':'reject_once'}]}})
            permission = json.loads(next(bounded_lines()))
            journal_record(journal, permission)
            permission = permission['result']['outcome'].get('optionId')
            release = None
            if '--release-native-init' in sys.argv:
                release = Path(sys.argv[sys.argv.index('--release-native-init') + 1])
            broken_order = '--python-before-native-control' in sys.argv
            if release and not broken_order:
                release.touch(mode=0o600)
            # The native server is this fixture's readiness precondition. The
            # relay deliberately advertises Runtime first; do not orient while
            # its required primary native context is still starting.
            ordered = servers if broken_order else sorted(servers, key=lambda s: s['name'] == 'bboxy_python')
            tools = {}
            for config in ordered:
                tools[config['name']] = call_server(config, journal)
                if release and broken_order and config['name'] == 'bboxy_python':
                    release.touch(mode=0o600)
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
