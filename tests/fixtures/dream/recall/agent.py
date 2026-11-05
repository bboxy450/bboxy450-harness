# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Private stdio ACP witness; no tools, identity, network or model."""
import json
from pathlib import Path
import sys

capture, log = map(Path, sys.argv[1:])
for line in sys.stdin:
    request = json.loads(line)
    method = request.get("method")
    if "id" not in request:
        continue
    if method == "initialize":
        result = {"protocolVersion": 1, "agentCapabilities": {}, "agentInfo": {"name": "private-recall", "version": "1"}}
    elif method == "session/new":
        result = {"sessionId": "private-recall-session"}
    elif method == "session/prompt":
        events = [json.loads(line) for line in log.read_text().splitlines()]
        assert any(e.get("event", {}).get("body", {}).get("kind") == "recall" for e in events), "reference must already be durable"
        capture.write_text(json.dumps(request["params"]))
        result = {"stopReason": "end_turn"}
    else:
        result = {}
    print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)
