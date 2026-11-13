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
import linecache
import os
import socket
import sys
import tempfile
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
        self.message = data.get("error", {}).get("message", str(self.code))
        detail = data.get("error", {})
        nested = detail.get("data")
        self.unknown = (detail.get("unknown", False) or data.get("status") == "unknown"
                        or (isinstance(nested, dict) and nested.get("status") == "unknown"))
        self.hints = data.get("hints", [])
        self.request_op_id = result.get("request_op_id")
        self.server, self.tool = result.get("server"), result.get("tool")
        super().__init__(json.dumps(result, ensure_ascii=False))

    def __str__(self):
        where = ".".join(part for part in (self.server, self.tool) if part)
        said = f"{where + ': ' if where else ''}{self.code}: {self.message}"
        for hint in self.hints[:3]:
            if isinstance(hint, dict) and hint.get("reason"):
                said += f"\n  hint: {hint['reason']}"
        return said + "\n  (error.result holds the server's full reply)"


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


class _Ready:
    """A finished result that `await` also accepts, so a forgotten or an extra
    `await` are both fine: `r = bboxy.orient()` and `r = await bboxy.orient()`."""
    def __await__(self):
        return self
        yield


class Data(Object, _Ready):
    pass


class Rows(list, _Ready):
    pass


class Text(str, _Ready):
    pass


def _ready(value):
    value = _objects(value)
    if isinstance(value, dict):
        return Data(value)
    if isinstance(value, list):
        return Rows(value)
    return Text(value if isinstance(value, str) else json.dumps(value))


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
    _catalogue = {}

    async def servers(self):
        return await _rpc("servers", {})

    async def _all(self, server):
        """Every tool definition of one server, read once per run."""
        if server not in self._catalogue:
            found, cursor = [], None
            while True:
                page = await self.tools(server, cursor)
                found.extend(page["tools"])
                cursor = page.get("next_cursor")
                if not cursor:
                    break
            self._catalogue[server] = found
        return self._catalogue[server]

    async def search(self, query, *, server=None, limit=8):
        """Find tools by words in their name or description, best first.

        Each match is `{server, name, signature, description}`; call it as
        `await tools.<server>.<name>(...)`.
        """
        words = [word for word in query.lower().replace("_", " ").split() if word]
        names = [server] if server else [item["name"] for item in await self.servers() if item.get("ready")]
        scored = []
        for name in names:
            for tool in await self._all(name):
                title = tool["name"].lower().replace("_", " ")
                text = f"{title} {tool.get('description', '').lower()}"
                score = sum(3 * (word in title) + (word in text) for word in words)
                if score:
                    scored.append((-score, name, tool["name"], tool))
        scored.sort(key=lambda row: row[:3])
        return [Object({"server": name, "name": tool["name"], "signature": _signature(tool),
                        "description": tool.get("description", "")})
                for _, name, _, tool in scored[:limit]]

    def __getattr__(self, server):
        """`tools.<server>` is that server's tools as functions."""
        if server.startswith("_"):
            raise AttributeError(server)
        return Server(server)

    async def tools(self, server, cursor=None):
        return await _rpc("tools", {"server": server, "cursor": cursor})

    async def call(self, server, tool, arguments, op_id=None, *, timeout_ms=None):
        return await asyncio.to_thread(self._call_sync, server, tool, arguments, op_id, timeout_ms)

    def _call_sync(self, server, tool, arguments, op_id=None, timeout_ms=None):
        response = _rpc_sync("call", {"server": server, "tool": tool,
                                      "arguments": arguments, "op_id": op_id, "timeout_ms": timeout_ms})
        result = response["result"]
        # Keep the exact result as .raw. .structured is only an alias; content,
        # refs, etags, metadata, media and structuredContent remain untouched.
        if result.get("isError"):
            error = ToolError(result)
            error.server, error.tool = server, tool
            error.request_op_id = response["request_op_id"]
            raise error
        return MCPResult(result, response["request_op_id"])


def _signature(tool):
    """`name(required: type, optional: type = ...)` from a tool's input schema."""
    schema = tool.get("inputSchema") or {}
    required = schema.get("required") or []
    fields = []
    for name, spec in sorted((schema.get("properties") or {}).items(), key=lambda item: item[0] not in required):
        kind = spec.get("type") if isinstance(spec, dict) else None
        if isinstance(spec, dict) and "enum" in spec:
            kind = " | ".join(json.dumps(choice) for choice in spec["enum"])
        kind = {"string": "str", "integer": "int", "number": "float", "boolean": "bool",
                "array": "list", "object": "dict"}.get(kind, kind or "any")
        fields.append(f"{name}: {kind}" if name in required else f"{name}: {kind} = ...")
    return f"{tool['name']}({', '.join(fields)})"


class Server:
    """One MCP server's tools as functions.

        page = bboxy.knowledge_search(query="type:task status:open")
        task = bboxy.task_status({"task": page.items[0].ref.slug})

    Arguments are keywords or one positional dict, passed to the tool unchanged.
    `await` is optional: the call has finished when it returns. For calls that
    should overlap, use `asyncio.gather` over `mcp.call(...)`.
    The call returns the tool's data (`MCPResult.data`); a tool error raises
    ToolError with the server's own code, message and hints. `_timeout_ms=` bounds
    this one call and `_op_id=` names its receipt; neither reaches the tool.
    """
    def __init__(self, name):
        self._name = name

    def __repr__(self):
        return f"<mcp server {self._name}: await {self._name}.tools() lists its functions>"

    async def tools(self):
        """Every tool of this server as `{name, signature, description}`."""
        return [Object({"name": tool["name"], "signature": _signature(tool),
                        "description": tool.get("description", "")})
                for tool in await mcp._all(self._name)]

    async def describe(self, tool):
        """One tool's signature, description and exact input schema."""
        for found in await mcp._all(self._name):
            if found["name"] == tool:
                return Object({"server": self._name, "name": tool, "signature": _signature(found),
                               "description": found.get("description", ""),
                               "input_schema": found.get("inputSchema")})
        raise AttributeError(f"{self._name} has no tool {tool!r}; await {self._name}.tools() lists them")

    def __getattr__(self, tool):
        if tool.startswith("_"):
            raise AttributeError(tool)

        def function(arguments=None, /, *, _op_id=None, _timeout_ms=None, **fields):
            if arguments is not None and fields:
                raise TypeError("pass arguments as keywords or as one dict, not both")
            if arguments is not None and not isinstance(arguments, dict):
                raise TypeError(f"{self._name}.{tool} takes keywords or one dict of arguments")
            return _ready(mcp._call_sync(self._name, tool, fields if arguments is None else arguments,
                                         _op_id, _timeout_ms).data)
        function.__name__ = function.__qualname__ = tool
        return function


class MCPResult(Object):
    def __init__(self, result, operation):
        super().__init__(result)
        self.request_op_id = operation

    @property
    def text(self):
        """The result's text blocks joined."""
        return "".join(block.get("text", "") for block in self.get("content") or []
                       if isinstance(block, dict) and block.get("type") == "text")

    @property
    def data(self):
        """What the tool returned: structuredContent, else its text parsed as JSON, else the text."""
        structured = self.get("structuredContent")
        if structured is not None:
            return structured
        try:
            return _objects(json.loads(self.text))
        except ValueError:
            return self.text

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


class Shell(Object, _Ready):
    """What `sh` returns: `.out`, `.err`, `.code`, `.ok`, `.json()`."""
    @property
    def out(self):
        return base64.b64decode(self["stdout_base64"]).decode(errors="replace")

    @property
    def err(self):
        return base64.b64decode(self["stderr_base64"]).decode(errors="replace")

    @property
    def code(self):
        return self["exit_code"]

    @property
    def ok(self):
        return self["exit_code"] == 0

    def json(self):
        return json.loads(self.out)

    def __str__(self):
        return self.out


def sh(command, *, check=False, input=None, cwd=None, timeout_ms=None):
    """Run a shell command and return its result at once.

        status = sh("git status --porcelain").out
        sh("cargo fmt --check", check=True)      # raises with stderr on a non-zero exit
    """
    if not isinstance(command, str):
        raise TypeError("sh takes one command string; for an argv list use commands.run([...])")
    encoded = None if input is None else base64.b64encode(
        input if isinstance(input, bytes) else str(input).encode()).decode()
    result = Shell(_rpc_sync("command", {
        "argv": ["/bin/sh", "-c", command], "cwd": os.fspath(cwd) if cwd else None,
        "input_base64": encoded, "timeout_ms": timeout_ms,
    }))
    if check and not result.ok:
        raise RuntimeError(f"`{command}` exited {result.code}\n{result.err or result.out}".rstrip())
    return result


class Files:
    """Small file helpers for the edits scripts do most."""
    def lines(self, path, start=1, end=None):
        """Numbered lines `start..end` of a text file, as one string."""
        rows = open(path, encoding="utf-8", errors="replace").read().splitlines()
        end = len(rows) if end is None else min(end, len(rows))
        return Text("".join(f"{number}\t{rows[number - 1]}\n" for number in range(max(start, 1), end + 1)))

    def replace(self, path, old, new, *, count=1):
        """Replace `old` with `new` in a file, exactly `count` times (None: every one).

        Raises without writing when `old` occurs a different number of times, and
        says how many it found and where the closest line is.
        """
        text = open(path, encoding="utf-8").read()
        found = text.count(old)
        if found == 0 or (count is not None and found != count):
            first = old.strip().splitlines()[0] if old.strip() else ""
            near = [f"{number}: {line.strip()}" for number, line in enumerate(text.splitlines(), 1)
                    if first and (first[:24] in line or line.strip()[:24] in first) and line.strip()][:3]
            raise ValueError(f"{path}: expected {count if count is not None else 'at least 1'} occurrence(s) "
                             f"of the old text, found {found}; nothing was written"
                             + ("; closest lines: " + " | ".join(near) if near and not found else ""))
        with open(path, "w", encoding="utf-8") as output:
            output.write(text.replace(old, new) if count is None else text.replace(old, new, count))
        return found if count is None else count


# The host keeps at most 64 KiB of emitted values for one run.
_EMIT_ROOM = [56 * 1024]


def emit(value):
    """Return a JSON value from the run. A value that no longer fits inline is
    written to a file and replaced by `{spilled, bytes, head}`: the run never
    fails because its answer was large."""
    data = json.dumps(value, ensure_ascii=False, default=str)
    if len(data.encode()) > _EMIT_ROOM[0]:
        directory = os.path.join(tempfile.gettempdir(), "bboxy-python", _execution)
        os.makedirs(directory, exist_ok=True)
        path = os.path.join(directory, f"emit-{len(os.listdir(directory))}.json")
        with open(path, "w", encoding="utf-8") as output:
            output.write(data)
        value = {"spilled": path, "bytes": len(data.encode()), "head": data[:1000]}
    else:
        value = json.loads(data)
    _EMIT_ROOM[0] -= len(json.dumps(value, ensure_ascii=False).encode()) + 16
    _rpc_sync("emit", {"value": value})


mcp = MCP()
tools = mcp
# The host's own tools are plain functions: `await bboxy.orient()`.
bboxy = Server("bboxy")
commands = Commands()
fs = Files()
inputs = _objects(_launch.get("inputs", {}))
execution = _objects({**_launch["context"], "execution_id": _execution})
workspace = execution.get("workspace")
# Regular imports from the actual source checkout work just as local scripts do.
sys.path.insert(0, os.getcwd())

_NAME = "<python_execute>"


async def _main():
    source = _launch["source"]
    linecache.cache[_NAME] = (len(source), None, source.splitlines(True), _NAME)
    tree = ast.parse(source, _NAME, "exec")
    # A final bare expression is the run's result, as in a notebook cell.
    last = tree.body[-1] if tree.body else None
    if isinstance(last, ast.Expr):
        tree.body[-1] = ast.copy_location(
            ast.Assign(targets=[ast.Name(id="_", ctx=ast.Store())], value=last.value), last)
        ast.fix_missing_locations(tree)
    globals().pop("_", None)
    result = eval(compile(tree, _NAME, "exec", flags=ast.PyCF_ALLOW_TOP_LEVEL_AWAIT), globals(), globals())
    if inspect.isawaitable(result):
        await result
    if isinstance(last, ast.Expr) and globals().get("_") is not None:
        emit(globals()["_"])


def _report(error):
    """The traceback from the script's own first frame down, not the worker's."""
    trace = error.__traceback__
    while trace is not None and trace.tb_frame.f_code.co_filename != _NAME:
        trace = trace.tb_next
    traceback.print_exception(type(error), error, trace or error.__traceback__)


try:
    asyncio.run(_main())
except SystemExit as stop:
    # sys.exit() or sys.exit(0) ends a script that is done.
    sys.exit(stop.code)
except ToolError as error:
    # A private callback preserves structure without parsing user stderr.
    # The receipt is observational; never retry the original nested operation.
    try:
        _rpc_sync("failure", {"failure": {
            "error": {"code": error.code, "message": error.message,
                      "op_id": error.request_op_id, "server": error.server,
                      "tool": error.tool, "unknown": error.unknown},
            "request_op_id": error.request_op_id, "server": error.server,
            "tool": error.tool, "hints": error.hints,
        }})
    except BaseException:
        pass
    _report(error)
    sys.exit(1)
except BaseException as error:
    _report(error)
    sys.exit(1)
