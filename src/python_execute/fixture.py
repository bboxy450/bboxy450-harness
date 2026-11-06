# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Real stdio MCP test peer; never uses a model, network or operator account."""
import json
import os
import sys
from pathlib import Path

tag = sys.argv[1]
root = Path(sys.argv[2])
names = ["echo", "fail", "write", "crash", "change", "change_silent"]
names += [f"tool_{i:03}" for i in range(180)]
if tag == "native":
    names = ["python_execution_context"]
generation = 0
aba_phase = None
for line in sys.stdin:
    request = json.loads(line)
    if "id" not in request:
        continue
    method = request["method"]
    if method == "initialize":
        result = {"protocolVersion": request["params"]["protocolVersion"],
                  "capabilities": {"tools": {"listChanged": True}},
                  "serverInfo": {"name": "bboxy450" if tag == "native" else tag, "version": "1"}}
    elif method == "tools/list":
        cursor = int(request.get("params", {}).get("cursor", "0"))
        if aba_phase == "fresh_a" and cursor == 0:
            (root / f"{tag}.aba-fresh-list-held").write_text("A refresh waiting")
            deadline = __import__('time').monotonic() + 5
            while not (root / f"{tag}.aba-release").exists():
                if __import__('time').monotonic() >= deadline:
                    raise RuntimeError("fixture A refresh release was never supplied")
                __import__('time').sleep(0.005)
            aba_phase = None
        selected = names[cursor:cursor + 70]
        tools = [{"name": n, "description": f"{tag}:{n}:{generation}",
                  "inputSchema": {"type": "object", "properties": {
                      "op_id": {"type": "integer"}, "payload": {}, "path": {"type": "string"}}},
                  "outputSchema": {"type": "object", "additionalProperties": True},
                  "annotations": {"readOnlyHint": n != "write"}} for n in selected]
        result = {"tools": tools}
        if cursor + len(selected) < len(names):
            result["nextCursor"] = str(cursor + len(selected))
        if aba_phase == "list_b" and cursor == 0:
            # Return the captured B list only after a newer A notification.
            # Holding the next fresh list lets Rust inspect the cache after
            # this stale listing finishes, without timing-dependent polling.
            generation += 1
            names.remove("transient_b")
            aba_phase = "fresh_a"
            print(json.dumps({"jsonrpc": "2.0", "method": "notifications/tools/list_changed"}), flush=True)
            __import__('time').sleep(0.05)
    elif method == "tools/call":
        name = request["params"]["name"]
        args = request["params"].get("arguments", {})
        if name == "echo" and args.get("payload") == {"catalogue_aba": True}:
            generation += 1
            names.append("transient_b")
            aba_phase = "list_b"
            print(json.dumps({"jsonrpc": "2.0", "method": "notifications/tools/list_changed"}), flush=True)
        if name == "echo" and args.get("payload") == {"timeout": True}:
            with (root / f"{tag}.timeout-dispatches").open("a") as output:
                output.write("dispatch\n")
            __import__('time').sleep(0.5)
        if name == "echo" and args.get("payload") == {"wait": True}:
            (root / f"{tag}.waiting").write_text("waiting")
            __import__('time').sleep(60)
        if name == "echo" and args.get("payload") == {"rpc_error": True}:
            print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "error": {
                "code": -32602, "message": "fixture argument correction", "data": {
                    "hints": [{"reason": "read original schemas", "method": "tools/list", "params": {}}]}}}), flush=True)
            continue
        if name == "echo" and args.get("payload") == {"rpc_unknown": True}:
            print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "error": {
                "code": -32001, "message": "native transport uncertainty", "data": {
                    "status": "unknown", "op_id": args["op_id"],
                    "hints": [{"reason": "reconcile original operation", "method": "tools/call",
                               "params": {"name": "operation_status", "arguments": {"op_id": args["op_id"]}}}]}}}), flush=True)
            continue
        if name == "echo" and args.get("payload") == {"result_unknown": True}:
            structured = {"status": "unknown", "op_id": args["op_id"],
                          "error": {"code": "unknown", "message": "native effect is uncertain"},
                          "hints": [{"reason": "reconcile original operation", "method": "tools/call",
                                     "params": {"name": "operation_status", "arguments": {"op_id": args["op_id"]}}}]}
            result = {"content": [{"type": "text", "text": json.dumps(structured)}],
                      "structuredContent": structured, "isError": True}
            print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)
            continue
        if name == "python_execution_context":
            if args.get("payload") == {"refresh_role": True}:
                generation += 1
                names.append("native_oracle_only")
                print(json.dumps({"jsonrpc": "2.0", "method": "notifications/tools/list_changed"}), flush=True)
            context = json.loads((root / "context.json").read_text())
            transport_version = context.pop("_transport_guard_version", 1)
            result = {"content": [{"type": "text", "text": "context"}],
                      "structuredContent": {"context": context, "fixture_pid": os.getpid()}, "isError": False,
                      "_meta": {"bboxy450/guard_transport_version": transport_version}}
            print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)
            continue
        if name == "crash":
            sys.exit(0)
        if name == "write":
            path = root / f"{tag}.writes"
            with path.open("a") as output:
                output.write("write\n")
        if name in ("change", "change_silent"):
            generation += 1
            names.append(f"added_{generation}")
            if name == "change":
                print(json.dumps({"jsonrpc": "2.0", "method": "notifications/tools/list_changed"}), flush=True)
        structured = {"server": tag, "name": name, "fixture_pid": os.getpid(), "arguments": args, "ref": {"opaque": "keep-me"},
                      "etag": "opaque-etag", "hints": [{"reason": "preserve this hint", "method": "tools/call",
                          "params": {"name": "echo", "arguments": {"op_id": 42}}}]}
        if name == "fail":
            structured["error"] = {"code": "fixture_rejected", "message": "correct the payload"}
        result = {"content": [{"type": "text", "text": "fixture"},
                              {"type": "resource_link", "uri": "fixture://opaque", "name": "original"}],
                  "structuredContent": structured, "isError": name == "fail",
                  "_meta": {"preserved": True}}
    else:
        result = {}
    print(json.dumps({"jsonrpc": "2.0", "id": request["id"], "result": result}), flush=True)
