#!/usr/bin/env python3
# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Unpaid, isolated real ACP child: dynamic config, legacy, refused/unknown/reordered replies."""
import json
import sys
import time
scenario, record = sys.argv[1:]
model, effort = 'alpha', 'low'

def send(message):
    print(json.dumps(message), flush=True)

def options():
    efforts = ['low', 'high'] if model == 'alpha' else ['balanced', 'deep']
    return [dict(id='vendor-model', name='Model', category='model', type='select', currentValue=model,
                 options=[dict(group='discovered', name='Discovered', options=[dict(value=m, name=m.title()) for m in ['alpha','beta','gamma']])]),
            dict(id='compute', name='Reasoning effort', category='thought_level', type='select', currentValue=effort,
                 options=[dict(value=e, name=e.title()) for e in efforts]),
            dict(id='secret', name='Do not project', category='mode', type='select', currentValue='PRIVATE-UNPROJECTED', options=[])]

for line in sys.stdin:
    msg = json.loads(line)
    with open(record, 'a', encoding='utf8') as out:
        out.write(json.dumps(msg) + '\n')
    method, params = msg.get('method'), msg.get('params', {})
    if method == 'initialize':
        result = dict(protocolVersion=1, agentCapabilities={}, agentInfo=dict(name='private-config-fixture', version='1'))
    elif method == 'session/new':
        result = dict(sessionId='private-agent-session')
        if scenario == 'legacy':
            result['models'] = dict(currentModelId=model, availableModels=[dict(modelId=m, name=m.title()) for m in ['alpha','beta']])
        elif scenario != 'unsupported':
            result['configOptions'] = options()
    elif method == 'session/prompt':
        result = dict(stopReason='end_turn')
    elif method == 'session/set_model':
        model = params['modelId']
        result = {}
    elif method == 'session/set_config_option':
        if scenario == 'unknown':
            continue
        if params['configId'] == 'compute' and params['value'] == 'deep':
            send(dict(jsonrpc='2.0', id=msg['id'], error=dict(code=-32602, message='unsupported effort')))
            continue
        time.sleep(0.04)  # pending is a separately observable state
        if params['configId'] == 'vendor-model':
            model, effort = params['value'], 'balanced' if params['value'] != 'alpha' else 'low'
        else:
            effort = params['value']
        result = dict(configOptions=options())
        # Wrong-session notification must never change the catalogue.
        send(dict(jsonrpc='2.0', method='session/update', params=dict(sessionId='rogue', update=dict(sessionUpdate='config_option_update', configOptions=[]))))
        if params['value'] == 'gamma' or scenario == 'bootstrap-reorder':
            model, effort = 'alpha', 'low'
            send(dict(jsonrpc='2.0', method='session/update', params=dict(sessionId='private-agent-session', update=dict(sessionUpdate='config_option_update', configOptions=options()))))
            time.sleep(0.04)  # older gamma response arrives after newer alpha update
    elif method == 'session/cancel':
        continue
    else:
        send(dict(jsonrpc='2.0', id=msg.get('id'), error=dict(code=-32601, message='unsupported')))
        continue
    send(dict(jsonrpc='2.0', id=msg['id'], result=result))

    if method == 'session/prompt' and scenario == 'restart':
        with open(record, encoding='utf8') as source:
            prompts = sum(json.loads(line).get('method') == 'session/prompt' for line in source)
        if prompts == 1:
            sys.exit(0)
