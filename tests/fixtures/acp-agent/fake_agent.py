# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""A scripted ACP agent over stdio for tests/acp_agent.rs. argv[1] picks the scenario."""
import json
import sys

SCENARIO = sys.argv[1] if len(sys.argv) > 1 else "text"
SESSION = "fake-session-1"
next_id = 1000


def send(msg):
    sys.stdout.write(json.dumps(msg) + "\n")
    sys.stdout.flush()


def update(u):
    send({"jsonrpc": "2.0", "method": "session/update", "params": {"sessionId": SESSION, "update": u}})


def chunk(kind, text):
    update({"sessionUpdate": kind, "content": {"type": "text", "text": text}})


def read():
    line = sys.stdin.readline()
    if not line:
        sys.exit(0)
    return json.loads(line)


def request(method, params):
    """Send an agent->client request and wait for its answer (other messages are kept)."""
    global next_id
    next_id += 1
    rid = next_id
    send({"jsonrpc": "2.0", "id": rid, "method": method, "params": params})
    while True:
        m = read()
        if m.get("id") == rid and "method" not in m:
            return m
        if m.get("method") == "session/cancel":
            CANCELLED.append(True)


CANCELLED = []


def prompt(rid, text):
    if SCENARIO == "text":
        chunk("agent_thought_chunk", "Thinking about it.")
        chunk("agent_message_chunk", "Hello ")
        chunk("agent_message_chunk", "from the fake agent.")
        update({"sessionUpdate": "plan", "entries": [{"content": "say hello", "priority": "high", "status": "completed"}]})
        update({"sessionUpdate": "available_commands_update", "availableCommands": []})
        return "end_turn"
    if SCENARIO == "tool":
        # an unsupported client method must be answered with -32601, never left hanging
        r = request("fs/read_text_file", {"sessionId": SESSION, "path": "/etc/hosts"})
        code = (r.get("error") or {}).get("code")
        chunk("agent_message_chunk", "fs answered %s. " % code)
        update({"sessionUpdate": "tool_call", "toolCallId": "call-1", "title": "Run ls", "kind": "execute",
                "status": "pending", "rawInput": {"command": "ls"}})
        r = request("session/request_permission", {
            "sessionId": SESSION,
            "toolCall": {"toolCallId": "call-1", "title": "Run ls", "kind": "execute", "rawInput": {"command": "ls"}},
            "options": [
                {"optionId": "yes", "name": "Allow", "kind": "allow_once"},
                {"optionId": "always", "name": "Always", "kind": "allow_always"},
                {"optionId": "no", "name": "Reject", "kind": "reject_once"},
            ],
        })
        outcome = r["result"]["outcome"]
        if outcome.get("outcome") == "selected" and outcome.get("optionId") == "yes":
            update({"sessionUpdate": "tool_call_update", "toolCallId": "call-1", "status": "completed",
                    "content": [{"type": "content", "content": {"type": "text", "text": "note.txt"}}]})
            chunk("agent_message_chunk", "Ran it.")
        else:
            update({"sessionUpdate": "tool_call_update", "toolCallId": "call-1", "status": "failed",
                    "content": [{"type": "content", "content": {"type": "text", "text": "permission %s" % json.dumps(outcome)}}]})
            chunk("agent_message_chunk", "Not allowed.")
        return "end_turn"
    if SCENARIO == "cancel":
        chunk("agent_message_chunk", "Working")
        while not CANCELLED:
            m = read()
            if m.get("method") == "session/cancel":
                CANCELLED.append(True)
        chunk("agent_message_chunk", " (late update after cancel)")
        return "cancelled"
    if SCENARIO == "crash":
        chunk("agent_message_chunk", "About to fail")
        sys.stderr.write("fake agent: boom\n")
        sys.stderr.flush()
        sys.exit(3)
    if SCENARIO == "unknown":
        update({"sessionUpdate": "mystery_update", "x": 1})
        chunk("agent_message_chunk", "ok")
        return "end_turn"
    if SCENARIO == "malformed":
        sys.stdout.write("this is not json\n")
        sys.stdout.flush()
        read()
    return "end_turn"


while True:
    m = read()
    method, rid = m.get("method"), m.get("id")
    if method == "initialize":
        send({"jsonrpc": "2.0", "id": rid, "result": {"protocolVersion": 1, "agentCapabilities": {}, "authMethods": []}})
    elif method == "session/new":
        send({"jsonrpc": "2.0", "id": rid, "result": {"sessionId": SESSION}})
    elif method == "session/prompt":
        text = " ".join(b.get("text", "") for b in m["params"]["prompt"])
        stop = prompt(rid, text)
        send({"jsonrpc": "2.0", "id": rid, "result": {"stopReason": stop}})
    elif method == "session/cancel":
        CANCELLED.append(True)
