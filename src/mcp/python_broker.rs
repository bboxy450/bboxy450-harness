// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Exact server addresses and full MCP envelopes, independent of model display caps.
use super::*;
use sha2::{Digest, Sha256};
use std::sync::atomic::Ordering;

pub(crate) type Capture = BTreeMap<String, (u64, String)>;
/// Default deadline for one nested Python MCP call, independent of the durable
/// execution's wall deadline. A timeout never authorizes replay of its effect.
pub(crate) const NESTED_CALL_TIMEOUT_MS: u64 = 25_000;

fn same_generation(capture: &Capture, server: &str, ready: &Ready) -> bool {
    capture
        .get(server)
        .is_some_and(|(generation, digest)| *generation == ready.generation && !digest.is_empty())
}
/// Refresh only the existing connection. Per-server serialization prevents late
/// list replies from overwriting a newer catalogue; no new process is started.
pub(super) async fn refresh_catalogue(
    slot: &Slot,
    cancel: &CancellationToken,
    deadline: tokio::time::Instant,
) -> Result<(), Value> {
    let refresh = async {
        let _guard = slot.catalogue_refresh.lock().await;
        let epoch = *slot
            .catalogue_epoch
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        let current = slot.ready.load_full();
        let ready = current.as_ref().as_ref().ok_or_else(|| {
            failure(
                "server_unavailable",
                "configured MCP server is not ready",
                false,
            )
        })?;
        let tools = ready.peer.list_all_tools().await.map_err(|_| {
            failure(
                "catalogue_unavailable",
                "MCP tools/list refresh failed; no tool call sent",
                false,
            )
        })?;
        if slot.generation.load(Ordering::Acquire) != ready.generation
            || slot.ready.load().as_ref().is_none()
        {
            return Err(failure(
                "server_generation_changed",
                "MCP connection changed during refresh; no call sent",
                false,
            ));
        }
        let schema_digest = schema_id(&tools);
        // Notification admission and publication share a short synchronous
        // critical section; no IO or await occurs while holding it.
        let current_epoch = slot
            .catalogue_epoch
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if *current_epoch != epoch {
            return Err(failure(
                "catalogue_changed",
                "MCP catalogue changed during listing; stale snapshot discarded, no call sent",
                false,
            ));
        }
        let published = slot.ready.compare_and_swap(
            &current,
            Arc::new(Some(Ready {
                generation: ready.generation,
                peer: ready.peer.clone(),
                tools,
                schema_digest,
            })),
        );
        drop(current_epoch);
        if !Arc::ptr_eq(&published, &current) {
            return Err(failure(
                "server_generation_changed",
                "MCP connection changed before catalogue publication; no call sent",
                false,
            ));
        }
        slot.initialized.notify_waiters();
        Ok(())
    };
    tokio::select! { biased;
        _ = cancel.cancelled() => Err(failure("cancelled", "catalogue refresh cancelled; no call sent", false)),
        result = tokio::time::timeout_at(deadline, refresh) => result.unwrap_or_else(|_| Err(failure("timeout", "catalogue refresh deadline exceeded; no call sent", false))),
    }
}
fn native_server(slot: &Slot, ready: &Ready) -> bool {
    slot.origin == Origin::Launcher
        && ready.peer.peer_info().is_some_and(|info| {
            info.server_info
                .as_ref()
                .is_some_and(|server| server.name == "bboxy450")
        })
}
/// The ordinary native bridge is deliberately serial. A queued mutation or a
/// blocked reply must not prevent status/context reads needed to inspect or
/// cancel Python. Reuse the trusted launcher descriptor and normal MCP/process
/// custody, with a session-owned read-only lane bound to each primary generation.
pub(super) async fn supervise_context(
    primary: Arc<Slot>,
    control: Arc<Slot>,
    redactor: Set,
    output: mpsc::Sender<Internal>,
    cancel: CancellationToken,
) {
    loop {
        let generation = loop {
            let notified = primary.initialized.notified();
            let ready = primary.ready.load_full();
            if let Some(ready) = ready.as_ref().as_ref() {
                if !native_server(&primary, ready)
                    || !ready
                        .tools
                        .iter()
                        .any(|tool| tool.name == "python_execution_context")
                {
                    return;
                }
                break ready.generation;
            }
            if primary.state.load(Ordering::Acquire) == 3 {
                return;
            }
            tokio::select! {biased;_=cancel.cancelled()=>return,_=notified=>{}}
        };
        let mut children = TaskGroup::new(cancel.child_token());
        let slot = control.clone();
        let r = redactor.clone();
        let tx = output.clone();
        let stop = children.cancel.clone();
        children.spawn(async move {
            supervise(slot, r, tx, stop).await;
        });
        loop {
            let notified = primary.initialized.notified();
            let current = primary.ready.load_full();
            if current
                .as_ref()
                .as_ref()
                .is_none_or(|ready| ready.generation != generation)
            {
                break;
            }
            tokio::select! {biased;_=cancel.cancelled()=>break,_=notified=>{}}
        }
        let _ = children.cancel_and_join().await;
        if cancel.is_cancelled() {
            return;
        }
    }
}
pub(super) fn schema_id(tools: &[Tool]) -> String {
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(tools).unwrap_or_default())
    )
}
fn failure(code: &str, message: &str, unknown: bool) -> Value {
    json!({"code":code,"message":message,"unknown":unknown})
}
impl Manager {
    pub(crate) fn capture(&self) -> Capture {
        self.slots
            .iter()
            .map(|slot| {
                let current = slot.ready.load();
                let state = current
                    .as_ref()
                    .as_ref()
                    .map(|r| (r.generation, r.schema_digest.clone()))
                    .unwrap_or((slot.generation.load(Ordering::Acquire), String::new()));
                (slot.name.clone(), state)
            })
            .collect()
    }
    pub(crate) fn servers(&self) -> Value {
        json!(
            self.slots
                .iter()
                .map(|slot| {
                    let ready = slot.ready.load();
                    json!({"name":slot.name,"ready":ready.as_ref().is_some(),
                "state":(["starting","ready","failed","dead"][usize::from(slot.state.load(Ordering::Acquire))]),
                "generation":slot.generation.load(Ordering::Acquire),
                "tool_count":ready.as_ref().as_ref().map_or(0, |r|r.tools.len()),
                "origin":slot.origin.label()})
                })
                .collect::<Vec<_>>()
        )
    }
    pub(crate) fn tools_page(
        &self,
        capture: &Capture,
        server: &str,
        offset: usize,
    ) -> Result<Value, Value> {
        let slot = self
            .slots
            .iter()
            .find(|s| s.name == server)
            .ok_or_else(|| {
                failure(
                    "server_unavailable",
                    "server is not configured in this harness",
                    false,
                )
            })?;
        let current = slot.ready.load_full();
        let ready = current.as_ref().as_ref().ok_or_else(|| {
            failure(
                "server_unavailable",
                "configured MCP server is not ready",
                false,
            )
        })?;
        if !same_generation(capture, server, ready) {
            return Err(failure(
                "server_generation_changed",
                "MCP connection changed; start a new execution",
                false,
            ));
        }
        if offset > ready.tools.len() {
            return Err(failure(
                "invalid_cursor",
                "tool cursor is outside this catalogue",
                false,
            ));
        }
        let mut end = offset;
        let mut bytes = 0;
        while end < ready.tools.len() && end - offset < 64 {
            let size = serde_json::to_vec(&ready.tools[end])
                .map_err(|_| failure("invalid_schema", "tool schema could not be encoded", false))?
                .len();
            if end > offset && bytes + size > 2 * 1024 * 1024 {
                break;
            }
            if size > crate::python_execute::FRAME - 1024 {
                return Err(failure(
                    "schema_too_large",
                    "single tool schema exceeds the 8 MiB transport limit; request a bounded schema/resource from the original server",
                    false,
                ));
            }
            bytes += size;
            end += 1;
        }
        Ok(json!({"server":server,"generation":ready.generation,
            "tools":ready.tools[offset..end], "next_cursor":(end<ready.tools.len()).then(||end.to_string())}))
    }
    pub(crate) async fn raw_call(
        &self,
        capture: &Capture,
        server: &str,
        tool: &str,
        arguments: Value,
        operation: Option<ulid::Ulid>,
        cancel: &CancellationToken,
    ) -> Result<Value, Value> {
        self.guarded_call(capture, server, tool, arguments, operation, None, cancel)
            .await
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Exact MCP address/data and independent admission, receipt and cancellation inputs"
    )]
    pub(crate) async fn guarded_call(
        &self,
        capture: &Capture,
        server: &str,
        tool: &str,
        arguments: Value,
        operation: Option<ulid::Ulid>,
        expected: Option<&Value>,
        cancel: &CancellationToken,
    ) -> Result<Value, Value> {
        self.guarded_call_with_timeout(
            capture, server, tool, arguments, operation, expected, None, cancel,
        )
        .await
    }
    #[allow(
        clippy::too_many_arguments,
        reason = "Exact address, custody, deadline and cancellation are independent inputs"
    )]
    pub(crate) async fn guarded_call_with_timeout(
        &self,
        capture: &Capture,
        server: &str,
        tool: &str,
        arguments: Value,
        operation: Option<ulid::Ulid>,
        expected: Option<&Value>,
        timeout: Option<Duration>,
        cancel: &CancellationToken,
    ) -> Result<Value, Value> {
        let slot = self
            .slots
            .iter()
            .find(|s| s.name == server)
            .ok_or_else(|| {
                failure(
                    "server_unavailable",
                    "server is not configured in this harness",
                    false,
                )
            })?;
        let deadline = tokio::time::Instant::now()
            + timeout.unwrap_or_else(|| Duration::from_secs(slot.config.timeout_s));
        let mut current = slot.ready.load_full();
        // A catalogue miss is pre-dispatch. Refresh once, then send at most
        // one tool call. An error after dispatch is never retried here.
        if let Some(ready) = current.as_ref().as_ref()
            && same_generation(capture, server, ready)
            && !ready.tools.iter().any(|definition| definition.name == tool)
        {
            refresh_catalogue(
                slot,
                cancel,
                deadline.min(tokio::time::Instant::now() + Duration::from_secs(10)),
            )
            .await?;
            current = slot.ready.load_full();
        }
        let ready = current.as_ref().as_ref().ok_or_else(|| {
            failure(
                "server_unavailable",
                "configured MCP server is not ready",
                false,
            )
        })?;
        if !same_generation(capture, server, ready) {
            return Err(failure(
                "server_generation_changed",
                "MCP connection changed; no call sent",
                false,
            ));
        }
        let definition = ready
            .tools
            .iter()
            .find(|t| t.name.as_ref() == tool)
            .ok_or_else(|| {
                failure(
                    "tool_unavailable",
                    "tool is not in this server's current catalogue",
                    false,
                )
            })?;
        if !arguments.is_object() {
            return Err(failure(
                "invalid_arguments",
                "tool arguments must be an object",
                false,
            ));
        }
        let native = native_server(slot, ready)
            && ready
                .tools
                .iter()
                .any(|t| t.name == "python_execution_context");
        let runtime = slot.origin == Origin::Runtime;
        let mut params = json!({"name":tool,"arguments":self.redactor.value(arguments)});
        if let Some(id) = operation {
            if native {
                params["_meta"] = json!({"bboxy450/op_id":id});
            }
            // External op_id properties may be domain IDs, not bboxy journal
            // tokens. Only native bboxy and our own runtime contract permit it.
            if (native || runtime)
                && definition
                    .input_schema
                    .get("properties")
                    .and_then(|p| p.get("op_id"))
                    .is_some()
            {
                if params["arguments"]
                    .get("op_id")
                    .is_some_and(|supplied| supplied != &json!(id))
                {
                    return Err(failure(
                        "op_id_mismatch",
                        "explicit operation IDs disagree; no call sent",
                        false,
                    ));
                }
                params["arguments"]["op_id"] = json!(id);
            }
        }
        if (native || runtime)
            && let Some(expected) = expected
        {
            params["_meta"]["bboxy450/python_context"] = expected.clone();
        }
        let params: CallToolRequestParams = serde_json::from_value(params)
            .map_err(|_| failure("invalid_arguments", "invalid MCP request envelope", false))?;
        let request = ClientRequest::CallToolRequest(CallToolRequest::new(params));
        let sent = tokio::select! { biased;
            _ = tokio::time::sleep_until(deadline) => return Err(failure("timeout", "MCP call deadline exceeded; effect may have completed; inspect the original operation_status", true)),
            _ = cancel.cancelled() => return Err(failure("cancelled", "execution cancelled before dispatch", false)),
            _ = self.root.cancelled() => return Err(failure("session_closed", "harness session is closed", false)),
            r = ready.peer.send_request_with_option(request, PeerRequestOptions::with_timeout(deadline.saturating_duration_since(tokio::time::Instant::now()))) => r,
        }.map_err(|_|failure("transport_unknown", "MCP request outcome is unknown; do not replay the script", true))?;
        let request_id = sent.id.clone();
        let reply = tokio::select! { biased;
            _ = tokio::time::sleep_until(deadline) => {
                // Best effort cancellation does not establish whether the
                // original effect happened, and never dispatches it again.
                let _ = tokio::time::timeout(Duration::from_millis(20), ready.peer.notify_cancelled(CancelledNotificationParam::new(Some(request_id), Some("nested Python MCP call deadline exceeded".into())))).await;
                let mut error = failure("timeout", "MCP call deadline exceeded; effect may have completed; inspect the original operation_status", true);
                error["op_id"] = json!(operation);
                error["tool"] = json!(tool);
                if let Some(operation) = operation {
                    error["data"]["hints"] = json!([{"reason":"Inspect the original operation; do not replay the timed-out effect", "method":"tools/call", "params":{"name":"operation_status", "arguments":{"original_op_id":operation}}}]);
                }
                return Err(error);
            },
            _ = cancel.cancelled() => {
                let _ = tokio::time::timeout(Duration::from_millis(100), ready.peer.notify_cancelled(CancelledNotificationParam::new(Some(request_id), Some("python execution cancelled".into())))).await;
                return Err(failure("transport_unknown", "cancelled after MCP dispatch; the effect may have completed", true));
            },
            _ = self.root.cancelled() => return Err(failure("transport_unknown", "session closed after MCP dispatch", true)),
            r = sent.await_response() => r,
        };
        if slot.generation.load(Ordering::Acquire) != ready.generation {
            return Err(failure(
                "transport_unknown",
                "MCP server changed after dispatch; reconcile the original operation",
                true,
            ));
        }
        match reply {
            Ok(ServerResult::CallToolResult(result)) => serde_json::to_value(result).map(|value|self.redactor.value(value))
                .map_err(|_|failure("invalid_result", "cannot encode MCP result", true)),
            Err(rmcp::ServiceError::Timeout { .. }) => {
                let mut error = failure("timeout", "MCP call deadline exceeded; effect may have completed; inspect the original operation_status", true);
                error["op_id"] = json!(operation);
                error["tool"] = json!(tool);
                Err(error)
            },
            Err(rmcp::ServiceError::McpError(error)) => Err(self.redactor.value(json!({"code":"mcp_error","rpc_code":error.code,"message":error.message,"data":error.data,"unknown":false}))),
            _ => Err(failure("transport_unknown", "MCP response unavailable; do not replay the script", true)),
        }
    }
    pub(crate) async fn execution_context(
        &self,
        capture: &Capture,
        cancel: &CancellationToken,
    ) -> Result<Option<Value>, Value> {
        for slot in self.slots.iter().filter(|s| s.origin == Origin::Launcher) {
            let current = slot.ready.load_full();
            let Some(ready) = current
                .as_ref()
                .as_ref()
                .filter(|ready| native_server(slot, ready))
            else {
                continue;
            };
            let native = ready
                .tools
                .iter()
                .any(|t| t.name == "python_execution_context");
            if native {
                // Fence the effect lane's captured identity too. The independent
                // lane is only for this exact native context query; callers
                // cannot address it to dispatch a mutation or broaden authority.
                if !same_generation(capture, &slot.name, ready) {
                    return Err(failure(
                        "server_generation_changed",
                        "Native MCP connection changed; no new execution is admitted",
                        false,
                    ));
                }
                let control = slot.context.as_ref().ok_or_else(|| {
                    failure(
                        "native_context_unavailable",
                        "Native read lane is unavailable",
                        false,
                    )
                })?;
                let deadline = tokio::time::Instant::now() + Duration::from_millis(1500);
                while control.ready.load().as_ref().is_none() {
                    if control.state.load(Ordering::Acquire) == 3
                        || tokio::time::Instant::now() >= deadline
                    {
                        return Err(failure(
                            "context_pending",
                            "Native read lane is not ready; retry orientation before effectful execution",
                            false,
                        ));
                    }
                    tokio::select! {biased;_=cancel.cancelled()=>return Err(failure("cancelled","Native context query cancelled",false)),_=tokio::time::sleep(Duration::from_millis(5))=>{}}
                }
                let context_client = Manager {
                    slots: Arc::new(vec![control.clone()]),
                    notices: self.notices.clone(),
                    root: self.root.clone(),
                    redactor: self.redactor.clone(),
                };
                let reply = context_client
                    .raw_call(
                        &context_client.capture(),
                        &slot.name,
                        "python_execution_context",
                        json!({}),
                        None,
                        cancel,
                    )
                    .await?;
                if reply["isError"] == true {
                    return Err(
                        json!({"code":reply["structuredContent"]["error"]["code"].as_str().unwrap_or("native_context_unavailable"),"message":reply["structuredContent"]["error"]["message"].as_str().unwrap_or("native execution context was refused"),"result":reply,"unknown":false}),
                    );
                }
                let context = reply["structuredContent"]["context"].clone();
                if !context.is_object() || context["guard_version"] != 1 {
                    return Err(failure(
                        "native_context_unavailable",
                        "backend lacks the native execution guard; upgrade/restart the backend and reconnect this harness",
                        false,
                    ));
                }
                if reply["_meta"]["bboxy450/guard_transport_version"] != 1 {
                    return Err(failure(
                        "native_context_unavailable",
                        "MCP bridge cannot preserve guarded dispatch metadata; update term and reconnect this harness's MCP client. The supervisor need not restart",
                        false,
                    ));
                }
                return Ok(Some(context));
            }
            return Err(failure(
                "native_context_unavailable",
                "Native bboxy backend/bridge lacks guarded Python context; upgrade/restart it before execution",
                false,
            ));
        }
        if std::env::var_os("BBOXY_SEAT_ID").is_some() {
            Err(failure(
                "native_context_unavailable",
                "admitted bboxy execution context is unavailable; update/reconnect the native MCP server",
                false,
            ))
        } else {
            Ok(None)
        }
    }
}
