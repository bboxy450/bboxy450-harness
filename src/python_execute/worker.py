# Copyright 2026 Boxy (bboxy450) and contributors.
# SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
# Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
# License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
# Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
# Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
"""Private CPython worker. Control RPC never shares user stdout/stderr."""
import ast
import asyncio
import base64
import inspect
import json
import os
import socket
import sys
import traceback

_launch = json.loads(sys.stdin.readline())
_address = _launch["socket"]
_execution = _launch["execution_id"]
_secret = _launch["callback_token"]
_max_frame = 8 * 1024 * 1024
_UNSET = object()


class ToolError(RuntimeError):
    def __init__(self, result):
        self.result = result
        data = result.get("structuredContent", result)
        self.code = data.get("error", {}).get("code", "tool_error")
        self.hints = data.get("hints", [])
        self.request_op_id = result.get("request_op_id")
        self.server, self.tool = result.get("server"), result.get("tool")
        super().__init__(json.dumps(result, ensure_ascii=False))


class Object(dict):
    """Attribute convenience without changing the underlying JSON fields."""
    def __getattr__(self, name):
        try:
            return self[name]
        except KeyError:
            raise AttributeError(name) from None


def _objects(value):
    if isinstance(value, dict):
        return Object({k: _objects(v) for k, v in value.items()})
    if isinstance(value, list):
        return [_objects(v) for v in value]
    return value


def _rpc_sync(method, arguments):
    request = {"jsonrpc": "2.0", "id": 1, "method": "python/callback", "params": {
        "execution_id": _execution, "token": _secret,
        "method": method, "arguments": arguments,
    }}
    data = json.dumps(request, ensure_ascii=False).encode() + b"\n"
    if len(data) > _max_frame:
        raise ValueError("python callback request exceeds 8 MiB")
    with socket.socket(socket.AF_UNIX) as connection:
        connection.connect(_address)
        connection.sendall(data)
        with connection.makefile("rb") as reader:
            data = reader.readline(_max_frame + 1)
    if len(data) > _max_frame:
        raise ValueError("python callback response exceeds 8 MiB")
    reply = json.loads(data)
    if "error" in reply:
        raise ToolError(reply["error"].get("data", reply["error"]))
    return _objects(reply["result"])


async def _rpc(method, arguments):
    return await asyncio.to_thread(_rpc_sync, method, arguments)


class MCP:
    async def servers(self):
        return await _rpc("servers", {})

    async def tools(self, server, cursor=None):
        return await _rpc("tools", {"server": server, "cursor": cursor})

    async def call(self, server, tool, arguments, op_id=None):
        response = await _rpc("call", {"server": server, "tool": tool,
                                        "arguments": arguments, "op_id": op_id})
        result = response["result"]
        # Keep the exact result as .raw. .structured is only an alias; content,
        # refs, etags, metadata, media and structuredContent remain untouched.
        if result.get("isError"):
            error = ToolError(result)
            error.server, error.tool = server, tool
            error.request_op_id = response["request_op_id"]
            raise error
        return MCPResult(result, response["request_op_id"])


class MCPResult(Object):
    def __init__(self, result, operation):
        super().__init__(result)
        self.request_op_id = operation

    @property
    def raw(self):
        return dict(self)

    @property
    def structured(self):
        return self.get("structuredContent")

    @property
    def hints(self):
        data = self.get("structuredContent")
        return data.get("hints", []) if isinstance(data, dict) else []


class CommandResult(Object):
    @property
    def stdout_bytes(self):
        return base64.b64decode(self["stdout_base64"])

    @property
    def stderr_bytes(self):
        return base64.b64decode(self["stderr_base64"])

    def json(self):
        if self.get("stdout_truncated"):
            raise ValueError("command stdout exceeded the retained budget; stdout_resource contains only the retained prefix. Narrow the read or explicitly write an output file; do not replay mutating scripts")
        return json.loads(self.stdout_bytes)


class Commands:
    async def run(self, argv, *, cwd=None, input=None, json_input=_UNSET, timeout_ms=None, check=False):
        if not isinstance(argv, (list, tuple)) or not argv:
            raise TypeError("commands.run requires a nonempty argv list")
        if json_input is not _UNSET:
            if input is not None:
                raise TypeError("input and json_input are mutually exclusive")
            input = json.dumps(json_input, ensure_ascii=False)
        if isinstance(input, bytes):
            encoded = base64.b64encode(input).decode()
        elif input is None:
            encoded = None
        elif isinstance(input, str):
            encoded = base64.b64encode(input.encode()).decode()
        else:
            raise TypeError("input requires str, bytes or None; use json_input for JSON values")
        result = CommandResult(await _rpc("command", {
            "argv": [os.fspath(x) for x in argv], "cwd": os.fspath(cwd) if cwd else None,
            "input_base64": encoded, "timeout_ms": timeout_ms,
        }))
        if check and result["exit_code"] != 0:
            raise ToolError({"error": {"code": "command_failed"}, "command": result})
        return result

    async def shell(self, source, **kwargs):
        return await self.run(["/bin/sh", "-c", source], **kwargs)

    async def available(self, name):
        return await _rpc("available", {"name": name})

    def __getattr__(self, name):
        if name.startswith("_"):
            raise AttributeError(name)
        async def command(*args, **kwargs):
            return await self.run([name, *args], **kwargs)
        return command


def emit(value):
    _rpc_sync("emit", {"value": value})


mcp = MCP()
tools = mcp
commands = Commands()
inputs = _objects(_launch.get("inputs", {}))
execution = _objects({**_launch["context"], "execution_id": _execution})
workspace = execution.get("workspace")
# Regular imports from the actual source checkout work just as local scripts do.
sys.path.insert(0, os.getcwd())

async def _main():
    compiled = compile(_launch["source"], "<python_execute>", "exec",
                       flags=ast.PyCF_ALLOW_TOP_LEVEL_AWAIT)
    result = eval(compiled, globals(), globals())
    if inspect.isawaitable(result):
        await result


try:
    asyncio.run(_main())
except BaseException:
    traceback.print_exc()
    sys.exit(1)
