// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! MCP and private callback framing. User stdout is never interpreted as RPC.
use super::*;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

pub fn catalogue() -> Vec<Value> {
    let id = json!({"type":"string","pattern":"^[0-7][0-9A-HJKMNP-TV-Z]{25}$"});
    [
        ("python_execute","Run full CPython in your current source workspace. Waits up to wait_ms (default 1000, clamped to 25000 ms), returning short results inline and durable handles for long runs. Earlier effects are never replayed after lost output. Read python_execute_orient first.",json!({"source":{"type":"string","minLength":1,"maxLength":SOURCE},"inputs":{},"timeout_ms":{"type":"integer","minimum":10,"maximum":TIMEOUT},"wait_ms":{"type":"integer","minimum":0,"description":"Bounded result wait, clamped to 25000 ms; 0 returns immediately."},"op_id":id}),vec!["source"]),
        ("python_execution_get","Read your session's Python execution and bounded output pages, optionally waiting for completion (wait_ms, default 1000, cap 25000 ms). UTF-8 output has text only; otherwise decode base64 for exact bytes. child:N:OFFSET reads exact JSON result bytes for nested receipt N.",json!({"execution_id":id,"cursor":{"type":"string","maxLength":256},"wait_ms":{"type":"integer","minimum":0}}),vec!["execution_id"]),
        ("python_execution_cancel","Cancel new work and retire the exact owned Python/command children. Earlier writes may have completed; inspect receipts. No script replay or rollback.",json!({"execution_id":id,"op_id":id}),vec!["execution_id"]),
        ("python_execute_orient","Read Python readiness, source workspace, current MCP server discovery, argv/jq bindings, limits and recovery examples. No package installation is required.",json!({}),vec![]),
    ].into_iter().map(|(name,description,properties,required)|json!({"name":name,"description":description,
        "inputSchema":{"type":"object","properties":properties,"required":required,"additionalProperties":false},
        "outputSchema":{"type":"object","additionalProperties":true},
        "annotations":{"readOnlyHint":matches!(name,"python_execution_get"|"python_execute_orient"),"destructiveHint":matches!(name,"python_execute"|"python_execution_cancel"),"idempotentHint":name!="python_execute","openWorldHint":true}})).collect()
}
impl Service {
    pub(super) async fn serve(&self, stream: tokio::net::UnixStream) {
        let (read, write) = stream.into_split();
        let mut reader = BufReader::new(read);
        let (writer, mut outgoing) = mpsc::channel::<Value>(64);
        let stop = self.0.cancel.clone();
        let output = bbt_rt::handle().spawn(async move {
            let mut write = write;
            loop {
                let value =
                    tokio::select! {biased;_=stop.cancelled()=>break,value=outgoing.recv()=>value};
                let Some(value) = value else { break };
                let Ok(mut bytes) = serde_json::to_vec(&value) else {
                    continue;
                };
                bytes.push(b'\n');
                let sent =
                    tokio::select! {biased;_=stop.cancelled()=>break,r=write.write_all(&bytes)=>r};
                if sent.is_err() {
                    break;
                }
            }
        });
        let mut tasks = tokio::task::JoinSet::new();
        loop {
            let mut bytes = vec![];
            let mut bounded = (&mut reader).take((FRAME + 1) as u64);
            let line = tokio::select! {biased;_=self.0.cancel.cancelled()=>break,
            n=bounded.read_until(b'\n',&mut bytes)=>n};
            let Ok(n) = line else { break };
            if n == 0 || n > FRAME {
                break;
            }
            let Ok(request) = serde_json::from_slice::<Value>(&bytes) else {
                break;
            };
            let Some(request_id) = request.get("id").cloned() else {
                continue;
            };
            if tasks.len() >= 32 {
                break;
            }
            let service = self.clone();
            let writer = writer.clone();
            tasks.spawn(async move {
                let progress = request["params"]["_meta"].get("progressToken").filter(|v| v.is_string() || v.is_number()).cloned().map(|token|(token,writer.clone()));
                let result=match request["method"].as_str() {
                    Some("initialize")=>Ok(json!({"protocolVersion":request["params"]["protocolVersion"].as_str().unwrap_or("2025-11-25"),"capabilities":{"tools":{}},"serverInfo":{"name":"bboxy-python","version":env!("CARGO_PKG_VERSION")}})),
                    Some("ping")=>Ok(json!({})),
                    Some("tools/list")=>Ok(json!({"tools":catalogue()})),
                    Some("tools/call"|"python/proxy")=>{
                        let proxy=request["method"]=="python/proxy";
                        let name=request["params"]["name"].as_str().unwrap_or("");
                        let mut args=request["params"]["arguments"].clone();
                        if name=="python_execute" && args.is_object() && args.get("op_id").is_none() {args["op_id"]=json!(Ulid::new());}
                        let handle=if name=="python_execute" {args["op_id"].clone()} else {args["execution_id"].clone()};
                        match tokio::time::timeout(Duration::from_millis(MAX_WAIT_MS),service.dispatch(name,args,proxy.then(||request["params"]["context"].clone()),(!proxy).then(||request["params"]["_meta"].get("bboxy450/python_context").cloned()).flatten(),progress)).await {
                            Ok(value)=>Ok(value),
                            Err(_)=>{let mut error=fault("request_wait_timeout","MCP wait cap reached; inspect the retained execution handle before retrying any effect");error["execution_id"]=handle;error["retry_script"]=json!(false);Ok(envelope(error))}
                        }
                    },
                    Some("python/callback")=>service.callback(request["params"].clone()).await,
                    _=>Err(fault("method_not_found","unknown runtime protocol method")),
                };
                let reply=match result {Ok(value)=>json!({"jsonrpc":"2.0","id":request_id,"result":value}),Err(error)=>json!({"jsonrpc":"2.0","id":request_id,"error":{"code":-32000,"message":"Python runtime callback failed","data":error}})};
                if let Ok(mut bytes)=serde_json::to_vec(&reply) {
                    if bytes.len()>FRAME {
                        bytes=serde_json::to_vec(&json!({"jsonrpc":"2.0","id":request_id,"error":{"code":-32000,"message":"Python runtime response exceeds its frame limit","data":fault("response_too_large","Response exceeds 8 MiB; inspect the retained execution and page results instead of replaying source")}})).unwrap_or_default();
                    }
                    if let Ok(value)=serde_json::from_slice(&bytes) {let _=writer.send(value).await;}
                }
            });
            while tasks.try_join_next().is_some() {}
        }
        while tasks.join_next().await.is_some() {}
        drop(writer);
        let _ = output.await;
    }
    async fn callback(&self, params: Value) -> Result<Value, Value> {
        let execution = id(&params, "execution_id")?;
        let run = self
            .0
            .runs
            .lock()
            .await
            .get(&execution)
            .cloned()
            .ok_or_else(|| {
                fault(
                    "execution_not_found",
                    "callback does not belong to a live execution",
                )
            })?;
        if params["token"].as_str() != Some(&run.token) {
            return Err(fault("forbidden", "invalid execution callback admission"));
        }
        // Bound slow host work per execution, not the control connection. A
        // batch cannot consume another run's emit capacity or deny get/cancel.
        let _execution_callback = tokio::select! {biased;_=run.cancel.cancelled()=>return Err(fault("cancelled","callback cancelled before admission")),r=run.callbacks.clone().acquire_owned()=>r.map_err(|_|fault("session_closed","callback admission closed"))?};
        let _callback = tokio::select! {biased;_=run.cancel.cancelled()=>return Err(fault("cancelled","callback cancelled before admission")),r=self.0.callbacks.clone().acquire_owned()=>r.map_err(|_|fault("session_closed","callback admission closed"))?};
        // Failure reporting is observational and must survive a revoked grant.
        // Other callbacks still re-admit current authority before doing work.
        if params["method"] != "failure" {
            self.check_context(&run).await?;
        }
        let args = &params["arguments"];
        match params["method"].as_str() {
            Some("failure") => {
                let value = self.0.redactor.value(args["failure"].clone());
                if !value["error"]["code"].is_string() {
                    return Err(fault(
                        "invalid_arguments",
                        "failure requires a structured error code",
                    ));
                }
                let mut record = run.record.lock().await;
                if record.status != "running" {
                    return Err(fault(
                        "execution_retired",
                        "Terminal execution cannot report a failure",
                    ));
                }
                if serde_json::to_vec(&value).map_or(usize::MAX, |v| v.len()) > 64 * 1024 {
                    return Err(fault("output_limit", "failure budget is 64 KiB"));
                }
                record.error = Some(value);
                self.save_record(&record)
                    .await
                    .map_err(|_| fault("execution_storage", "cannot persist Python failure"))?;
                Ok(Value::Null)
            }
            Some("servers") => Ok(self.0.mcp.servers()),
            Some("tools") => {
                let server = args["server"]
                    .as_str()
                    .ok_or_else(|| fault("invalid_arguments", "server name required"))?;
                run.pin_starting_server(server, self.0.mcp.capture());
                let offset = args["cursor"]
                    .as_str()
                    .unwrap_or("0")
                    .parse::<usize>()
                    .map_err(|_| fault("invalid_cursor", "invalid MCP tool cursor"))?;
                self.0
                    .mcp
                    .tools_page(&run.capture(), server, offset)
                    .map_err(|e| json!({"error":e}))
            }
            Some("emit") => {
                let value = self.0.redactor.value(args["value"].clone());
                let mut record = run.record.lock().await;
                if record.status != "running" || run.cancel.is_cancelled() {
                    return Err(fault(
                        "execution_retired",
                        "Terminal executions do not admit new output",
                    ));
                }
                let bytes = serde_json::to_vec(&record.emitted).map_or(usize::MAX, |b| b.len())
                    + serde_json::to_vec(&value).map_or(usize::MAX, |b| b.len());
                if bytes > 64 * 1024 {
                    return Err(fault(
                        "output_limit",
                        "emit budget is 64 KiB; save a file and emit a reference",
                    ));
                }
                record.emitted.push(value);
                self.save_record(&record)
                    .await
                    .map_err(|_| fault("execution_storage", "cannot persist emitted result"))?;
                Ok(Value::Null)
            }
            Some("available") => {
                let name = args["name"]
                    .as_str()
                    .ok_or_else(|| fault("invalid_arguments", "command name required"))?;
                Ok(
                    json!({"name":name,"path":process::resolve_in(name,run.record.lock().await.cwd.clone()).await}),
                )
            }
            Some("call" | "command") => {
                let kind = params["method"].as_str().unwrap_or_default();
                if kind == "call"
                    && (!args["server"].is_string()
                        || !args["tool"].is_string()
                        || !args["arguments"].is_object())
                {
                    return Err(fault(
                        "invalid_arguments",
                        "MCP call requires exact server/tool strings and a JSON arguments object",
                    ));
                }
                let call_timeout = if kind == "call" {
                    let timeout = if args["timeout_ms"].is_null() {
                        crate::mcp::NESTED_CALL_TIMEOUT_MS
                    } else {
                        args["timeout_ms"].as_u64().ok_or_else(|| {
                            fault(
                                "invalid_arguments",
                                "nested MCP timeout_ms must be an integer",
                            )
                        })?
                    };
                    if !(10..=TIMEOUT).contains(&timeout) {
                        return Err(fault(
                            "invalid_arguments",
                            "nested MCP timeout_ms must be 10..600000",
                        ));
                    }
                    Some(Duration::from_millis(timeout))
                } else {
                    None
                };
                let operation = if args["op_id"].is_null() {
                    Ulid::new()
                } else {
                    id(args, "op_id")?
                };
                let binding = if kind == "call" {
                    let server = args["server"].as_str().unwrap_or_default();
                    run.pin_starting_server(server, self.0.mcp.capture());
                    run.capture().get(server).cloned()
                } else {
                    None
                };
                let sequence = {
                    let mut record = run.record.lock().await;
                    // This same record lock serializes finish/cancel with new
                    // effect admission, including callbacks already in flight.
                    if run.cancel.is_cancelled() || record.status != "running" {
                        return Err(fault(
                            "cancelled",
                            "execution no longer admits nested effects",
                        ));
                    }
                    if record.children.len() >= CALLS {
                        return Err(fault(
                            "nested_call_limit",
                            "execution reached its nested call budget",
                        ));
                    }
                    if record.children.iter().any(|c| c.id == operation) {
                        return Err(fault(
                            "child_op_id_reused",
                            "this child operation already has a receipt; inspect it instead of redispatching",
                        ));
                    }
                    let sequence = record.children.len();
                    record.children.push(Child {
                        id: operation,
                        sequence,
                        kind: kind.into(),
                        server: args["server"].as_str().map(str::to_owned),
                        tool: args["tool"].as_str().map(str::to_owned),
                        arguments_digest: digest(args),
                        server_generation: binding.as_ref().map(|b| b.0),
                        schema_digest: binding.map(|b| b.1),
                        status: "dispatching".into(),
                        result: None,
                        result_resource: None,
                    });
                    self.save_record(&record).await.map_err(|_| {
                        fault(
                            "execution_storage",
                            "cannot persist child identity before dispatch",
                        )
                    })?;
                    sequence
                };
                // Persist the child identity first, then re-admit. Native
                // dispatch also carries the capture into the owner turn that
                // selects its real provider port; a preflight alone is racy.
                let expected = self.check_context(&run).await;
                let mut result = if let Err(error) = &expected {
                    Err(error.clone())
                } else if kind == "call" {
                    let server = args["server"]
                        .as_str()
                        .ok_or_else(|| fault("invalid_arguments", "server name required"))?;
                    let tool = args["tool"]
                        .as_str()
                        .ok_or_else(|| fault("invalid_arguments", "tool name required"))?;
                    self.0
                        .mcp
                        .guarded_call_with_timeout(
                            &run.capture(),
                            server,
                            tool,
                            args["arguments"].clone(),
                            Some(operation),
                            expected.as_ref().ok(),
                            call_timeout,
                            &run.cancel,
                        )
                        .await
                        .map_err(|e| json!({"error":e}))
                } else {
                    self.command(&run, args.clone(), operation).await
                };
                if let Err(error) = &mut result {
                    error["request_op_id"] = json!(operation);
                    error["execution_id"] = json!(execution);
                    if kind == "call" {
                        error["server"] = args["server"].clone();
                        error["tool"] = args["tool"].clone();
                    }
                    if !error["hints"].is_array() && error["error"]["data"]["hints"].is_array() {
                        error["hints"] = error["error"]["data"]["hints"].clone();
                    }
                    if !error["hints"].is_array() {
                        error["hints"] = json!([{"reason":"Inspect the original nested receipt and outcome; never replay the script after uncertain effects.","method":"tools/call","params":{"name":"python_execution_get","arguments":{"execution_id":execution,"cursor":format!("child:{sequence}:0")}}}]);
                    }
                }
                let (status, value) = match &result {
                    // Native bboxy can return an admitted operation whose effect
                    // is uncertain in either MCP error form. Keep that custody
                    // state distinct from a confirmed rejection; retain the raw
                    // envelope and its original recovery hints unchanged.
                    Ok(value)
                        if value["isError"] == true
                            && value["structuredContent"]["status"] == "unknown" =>
                    {
                        ("unknown", value.clone())
                    }
                    Ok(value) if value["isError"] == true => ("failed", value.clone()),
                    Ok(value) => ("succeeded", value.clone()),
                    Err(value)
                        if value["error"]["unknown"] == true
                            || value["error"]["data"]["status"] == "unknown" =>
                    {
                        ("unknown", value.clone())
                    }
                    Err(value) => ("failed", value.clone()),
                };
                let bytes = serde_json::to_vec(&value)
                    .map_err(|_| fault("invalid_result", "cannot encode nested result"))?;
                let mut record = run.record.lock().await;
                let receipt = &mut record.children[sequence];
                receipt.status = status.into();
                if bytes.len() > 16 * 1024 {
                    let path = self.0.root.join(format!("{execution}.{sequence}.result"));
                    let mut file = tokio::fs::OpenOptions::new()
                        .create_new(true)
                        .write(true)
                        .mode(0o600)
                        .open(&path)
                        .await
                        .map_err(|_| fault("execution_storage", "cannot preserve nested result"))?;
                    file.write_all(&bytes)
                        .await
                        .map_err(|_| fault("execution_storage", "cannot preserve nested result"))?;
                    file.sync_all()
                        .await
                        .map_err(|_| fault("execution_storage", "cannot persist nested result"))?;
                    receipt.result_resource = Some(path);
                } else {
                    receipt.result = Some(value);
                }
                self.save_record(&record)
                    .await
                    .map_err(|_| fault("execution_storage", "cannot persist nested receipt"))?;
                drop(record);
                self.check_context(&run).await?;
                if bytes.len() > FRAME / 2 {
                    let mut error = fault(
                        "result_too_large",
                        &format!(
                            "Nested result is preserved at child:{sequence}:0; page it through python_execution_get"
                        ),
                    );
                    error["execution_id"] = json!(execution);
                    error["request_op_id"] = json!(operation);
                    error["result_cursor"] = json!(format!("child:{sequence}:0"));
                    return Err(error);
                }
                match result {
                    Ok(value) if kind == "call" => {
                        Ok(json!({"result":value,"request_op_id":operation}))
                    }
                    other => other,
                }
            }
            _ => Err(fault("method_not_found", "unknown Python binding")),
        }
    }
    async fn command(&self, run: &Arc<Run>, args: Value, operation: Ulid) -> Result<Value, Value> {
        let argv: Vec<String> = serde_json::from_value(args["argv"].clone())
            .map_err(|_| fault("invalid_arguments", "argv must be a nonempty string array"))?;
        let Some(program) = argv.first() else {
            return Err(fault("invalid_arguments", "argv must be nonempty"));
        };
        if argv.len() > 1024 || argv.iter().any(|s| s.contains('\0')) {
            return Err(fault("invalid_arguments", "invalid or oversized argv"));
        }
        let base = run.record.lock().await.cwd.clone();
        let cwd = match &args["cwd"] {
            Value::Null => base,
            Value::String(path) => {
                let path = PathBuf::from(path);
                if path.is_absolute() {
                    path
                } else {
                    base.join(path)
                }
            }
            _ => {
                return Err(fault(
                    "invalid_arguments",
                    "command cwd must be a path string or null",
                ));
            }
        };
        let executable = process::resolve_in(program, cwd.clone())
            .await
            .ok_or_else(|| {
                fault(
                    "command_unavailable",
                    &format!("executable is not available: {program}"),
                )
            })?;
        let mut command = tokio::process::Command::new(executable);
        command.args(&argv[1..]);
        let input = args["input_base64"]
            .as_str()
            .map(|s| STANDARD.decode(s))
            .transpose()
            .map_err(|_| fault("invalid_arguments", "invalid command input bytes"))?;
        let timeout = if args["timeout_ms"].is_null() {
            120_000
        } else {
            args["timeout_ms"].as_u64().ok_or_else(|| {
                fault("invalid_arguments", "command timeout_ms must be an integer")
            })?
        };
        if !(10..=TIMEOUT).contains(&timeout) {
            return Err(fault(
                "invalid_arguments",
                "command timeout must be 10..600000 ms",
            ));
        }
        let started = tokio::time::Instant::now();
        let output = self
            .capture(run, command, cwd.clone(), input, timeout, false)
            .await?;
        let save = |stream: &str, bytes: Vec<u8>| {
            let path = self.0.root.join(format!("{operation}.{stream}"));
            async move {
                let mut f = tokio::fs::OpenOptions::new()
                    .write(true)
                    .create_new(true)
                    .mode(0o600)
                    .open(&path)
                    .await
                    .map_err(|_| fault("execution_storage", "cannot preserve command output"))?;
                f.write_all(&bytes)
                    .await
                    .map_err(|_| fault("execution_storage", "cannot preserve command output"))?;
                f.sync_all()
                    .await
                    .map_err(|_| fault("execution_storage", "cannot persist command output"))?;
                Ok::<_, Value>(path)
            }
        };
        let stdout_resource = save("stdout", output.stdout.clone()).await?;
        let stderr_resource = save("stderr", output.stderr.clone()).await?;
        Ok(
            json!({"process_id":output.pid,"child_request_id":operation,"cwd":cwd,"exit_code":output.code,"signal":output.signal,
            "stdout":String::from_utf8_lossy(&output.stdout),"stderr":String::from_utf8_lossy(&output.stderr),
            "stdout_base64":STANDARD.encode(&output.stdout),"stderr_base64":STANDARD.encode(&output.stderr),
            "stdout_total_bytes":output.stdout_bytes,"stderr_total_bytes":output.stderr_bytes,"retained_limit_bytes":OUTPUT,
            "stdout_truncated":output.stdout_bytes>output.stdout.len() as u64,"stderr_truncated":output.stderr_bytes>output.stderr.len() as u64,
            "stdout_resource":stdout_resource,"stderr_resource":stderr_resource,"cancelled":output.cancelled,"timed_out":output.timed_out,"duration_ms":started.elapsed().as_millis()}),
        )
    }
}
