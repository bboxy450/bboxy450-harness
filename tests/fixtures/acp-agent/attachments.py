# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Private ACP child: records exact requests and delays prompts to exercise queueing."""
import json
import sys
import time
prompt_count = 0
record, supported = sys.argv[1], sys.argv[2] == 'yes'
for line in sys.stdin:
    message = json.loads(line)
    with open(record, 'a', encoding='utf8') as out:
        out.write(json.dumps(message, ensure_ascii=False) + '\n')
    method = message.get('method')
    if method == 'initialize':
        result = {'protocolVersion': 1, 'agentCapabilities': {'promptCapabilities': {'image': supported}}}
    elif method == 'session/new':
        result = {'sessionId': 'private-attachment-child'}
    elif method == 'session/prompt':
        prompt_count += 1
        time.sleep(2 if prompt_count == 1 else 0.05)
        result = {'stopReason': 'end_turn'}
    else:
        result = {}
    if 'id' in message:
        print(json.dumps({'jsonrpc': '2.0', 'id': message['id'], 'result': result}), flush=True)
