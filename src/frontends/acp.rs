// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Stable ACP v1 JSONL dispatcher; prompt responses never hold the reader.
use crate::{
    engine::{Config, task_group::TaskGroup},
    gateway::{Gateway, Host, InProc, Subscribe, Subscription, TryRecv},
    protocol::*,
    provider::profile::Unattended,
    store::Blobs,
};
use agent_client_protocol::schema::v1 as schema;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::{
    io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader},
    sync::mpsc,
};
use tokio_util::sync::CancellationToken;
#[derive(Default)]
struct Pending {
    ops: BTreeMap<OpId, Value>,
    turns: BTreeMap<TurnId, Value>,
}
struct Session {
    gateway: InProc,
    pending: Arc<Mutex<Pending>>,
    blobs: Blobs,
}
struct Permission {
    session: SessionId,
    item: ItemId,
    options: Vec<ApprovalDecisionKind>,
}
enum ConnectionInput {
    Message(Value),
    ParseError,
}
struct Face {
    host: Host,
    config: Config,
    sessions: Mutex<BTreeMap<SessionId, Arc<Session>>>,
    permissions: Mutex<BTreeMap<String, Permission>>,
    output: mpsc::Sender<Value>,
    drivers: Mutex<TaskGroup>,
    inputs: Mutex<BTreeMap<SessionId, mpsc::UnboundedSender<Value>>>,
    connection: mpsc::UnboundedSender<ConnectionInput>,
    cancel: CancellationToken,
    next: AtomicU64,
    unknown: AtomicU64,
}
fn admission_events(
    mut sub: Subscription,
) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = Envelope> + Send>> {
    Box::pin(async_stream::stream! {while let Some(event)=sub.recv().await {yield event;}})
}
async fn await_prompt_admission(
    gateway: &InProc,
    op_id: OpId,
    mut events: std::pin::Pin<Box<dyn futures_core::Stream<Item = Envelope> + Send>>,
    cancel: &CancellationToken,
) -> Option<Envelope> {
    use futures_util::StreamExt;
    loop {
        let event =
            tokio::select! {biased;_=cancel.cancelled()=>return None,event=events.next()=>event};
        let event = event?;
        if let Event::Lagged { resume_after } = event.event {
            // The first caused event can be beyond the subscriber's retained
            // tail. Recover it from durable history before the next input.
            let sub = gateway
                .subscribe(Subscribe {
                    session: gateway.session(),
                    after: Some((event.epoch, resume_after)),
                    ephemeral: false,
                })
                .ok()?;
            events = admission_events(sub);
            continue;
        }
        if event.cause == Some(op_id) {
            return Some(event);
        }
    }
}
impl Face {
    fn spawn(&self, future: impl std::future::Future<Output = ()> + Send + 'static) {
        let mut group = self.drivers.lock().unwrap();
        group.reap_ready();
        group.spawn(future);
    }
    /// Admission is FIFO within a session, independent of output backpressure.
    /// Loading and other sessions have their own workers; pending prompts park
    /// only their replies in the projection, never this input queue.
    fn route(self: &Arc<Self>, request: Value) {
        let sid = request["params"]["sessionId"]
            .as_str()
            .and_then(|s| s.parse().ok())
            .map(SessionId);
        if let Some(sid) = sid {
            let known = self.sessions.lock().unwrap().contains_key(&sid);
            let mut inputs = self.inputs.lock().unwrap();
            if known || inputs.contains_key(&sid) || request["method"] == "session/load" {
                let tx=inputs.entry(sid).or_insert_with(|| {
                    let (tx,mut rx)=mpsc::unbounded_channel();
                    let face=self.clone();
                    self.spawn(async move {loop {
                        let request=tokio::select!{biased;_=face.cancel.cancelled()=>break,r=rx.recv()=>r};
                        let Some(request)=request else {break;};
                        face.clone().dispatch(request).await;
                    }});
                    tx
                });
                let _ = tx.send(request);
                return;
            }
        }
        let _ = self.connection.send(ConnectionInput::Message(request));
    }
    async fn emit(&self, value: Value) {
        let value = self.config.redactor.value(value);
        let Some(value) = bound(value) else {
            return;
        };
        // Only admission/subscription tasks wait for bounded capacity. Queue
        // fullness has no expiry and never closes a connected peer's session.
        tokio::select! {biased;_=self.cancel.cancelled()=>{},r=self.output.send(value)=>if r.is_err(){self.cancel.cancel();}}
    }
    async fn response(&self, id: Value, result: Value) {
        self.emit(json!({"jsonrpc":"2.0","id":id,"result":result}))
            .await;
    }
    async fn error(&self, id: Value, code: i32, message: &str, data: Option<Value>) {
        let mut error = json!({"code":code,"message":message});
        if let Some(data) = data {
            error["data"] = data;
        }
        self.emit(json!({"jsonrpc":"2.0","id":id,"error":error}))
            .await;
    }
    async fn update(&self, session: SessionId, update: Value) {
        let value = json!({"jsonrpc":"2.0","method":"session/update","params":{"sessionId":session.to_string(),"update":update}});
        let Some(value) = bound(self.config.redactor.value(value)) else {
            return;
        };
        tokio::select! {biased;_=self.cancel.cancelled()=>{},result=self.output.send(value)=>if result.is_err(){self.cancel.cancel();}}
    }
    fn session(&self, params: &Value) -> Option<Arc<Session>> {
        let id = params["sessionId"].as_str()?.parse().ok()?;
        self.sessions.lock().unwrap().get(&SessionId(id)).cloned()
    }
    fn send(
        &self,
        session: &Session,
        op: Op,
    ) -> Result<crate::gateway::Ack, crate::gateway::Refused> {
        session.gateway.send(OpEnvelope {
            v: PROTOCOL,
            session: session.gateway.session(),
            op_id: OpId::new(),
            op,
        })
    }
    async fn dispatch(self: Arc<Self>, request: Value) {
        if request["jsonrpc"] != "2.0" || !request.is_object() {
            self.error(request["id"].clone(), -32600, "Invalid Request", None)
                .await;
            return;
        }
        let Some(method) = request["method"].as_str() else {
            self.permission_response(request).await;
            return;
        };
        let id = request.get("id").cloned();
        let params = request.get("params").cloned().unwrap_or(json!({}));
        if id.is_none() {
            if method == "session/cancel" {
                if let Some(session) = self.session(&params) {
                    let _ = self.send(&session, Op::Interrupt { turn: None });
                }
            } else {
                self.unknown.fetch_add(1, Ordering::Relaxed);
            }
            return;
        }
        let id = id.unwrap();
        match method {
            "initialize" => {
                if serde_json::from_value::<schema::InitializeRequest>(params).is_err() {
                    self.error(id, -32602, "Invalid params", None).await;
                    return;
                }
                self.response(id,json!({"protocolVersion":1,"authMethods":[],"agentCapabilities":{"loadSession":true,"promptCapabilities":{"image":self.config.provider.caps().images,"audio":false,"embeddedContext":true},"mcpCapabilities":{"http":false,"sse":false},"sessionCapabilities":{"close":{}}},"agentInfo":{"name":"bboxy450-harness","version":env!("CARGO_PKG_VERSION")}})).await;
            }
            "session/new" | "session/load" => self.load(method == "session/load", id, params).await,
            "session/prompt" => {
                if !params["prompt"].is_array()
                    || serde_json::from_value::<schema::PromptRequest>({
                        let mut p = params.clone();
                        p["prompt"] = json!([]);
                        p
                    })
                    .is_err()
                {
                    self.error(id, -32602, "Invalid params", None).await;
                    return;
                }
                let Some(session) = self.session(&params) else {
                    self.error(id, -32002, "Unknown session", None).await;
                    return;
                };
                match self.input(&session, &params["prompt"]).await {
                    Ok(blocks) => {
                        let op_id = OpId::new();
                        let admission = session
                            .gateway
                            .subscribe(Subscribe {
                                session: session.gateway.session(),
                                after: Some((
                                    session.gateway.hello().epoch,
                                    session.gateway.hello().head,
                                )),
                                ephemeral: false,
                            })
                            .expect("loaded session");
                        session
                            .pending
                            .lock()
                            .unwrap()
                            .ops
                            .insert(op_id, id.clone());
                        if let Err(reason) = session.gateway.send(OpEnvelope {
                            v: PROTOCOL,
                            session: session.gateway.session(),
                            op_id,
                            op: Op::UserTurn {
                                blocks,
                                mode: TurnMode::Queue,
                            },
                        }) {
                            session.pending.lock().unwrap().ops.remove(&op_id);
                            self.error(
                                id,
                                -32603,
                                "Prompt admission refused",
                                Some(json!({"reason":reason})),
                            )
                            .await;
                        } else {
                            // Ack is lane admission, not durable consumption. The
                            // control lane drains first, so a following cancel
                            // must wait for this prompt's first durable outcome.
                            await_prompt_admission(
                                &session.gateway,
                                op_id,
                                admission_events(admission),
                                &self.cancel,
                            )
                            .await;
                        }
                    }
                    Err(message) => self.error(id, -32602, &message, None).await,
                }
            }
            "session/close" => {
                if let Some(session) = self.session(&params) {
                    let _ = self.send(&session, Op::Close);
                    self.response(id, json!({})).await;
                } else {
                    self.error(id, -32002, "Unknown session", None).await;
                }
            }
            _ => self.error(id, -32601, "Method not found", None).await,
        }
    }
    async fn load(self: &Arc<Self>, load: bool, id: Value, params: Value) {
        let valid = if load {
            serde_json::from_value::<schema::LoadSessionRequest>(params.clone()).is_ok()
        } else {
            serde_json::from_value::<schema::NewSessionRequest>(params.clone()).is_ok()
        };
        if !valid {
            self.error(id, -32602, "Invalid params", None).await;
            return;
        }
        let cwd = PathBuf::from(params["cwd"].as_str().unwrap_or(""));
        let session_id = if load {
            match params["sessionId"].as_str().and_then(|s| s.parse().ok()) {
                Some(id) => SessionId(id),
                None => {
                    self.error(id, -32602, "Invalid session id", None).await;
                    return;
                }
            }
        } else {
            SessionId::new()
        };
        if self.sessions.lock().unwrap().contains_key(&session_id) {
            self.error(id, -32602, "Session already attached", None)
                .await;
            return;
        }
        let result = if load {
            self.host.open(session_id).await
        } else {
            let mut overrides = crate::mcp::Servers::new();
            for server in params["mcpServers"].as_array().into_iter().flatten() {
                let Some(name) = server["name"].as_str() else {
                    continue;
                };
                if let Some(command) = server["command"].as_str() {
                    let mut env = BTreeMap::new();
                    for entry in server["env"].as_array().into_iter().flatten() {
                        if let (Some(name), Some(value)) =
                            (entry["name"].as_str(), entry["value"].as_str())
                        {
                            env.insert(name.into(), value.into());
                        }
                    }
                    overrides.insert(
                        name.into(),
                        crate::mcp::ServerConfig {
                            command: command.into(),
                            args: server["args"]
                                .as_array()
                                .into_iter()
                                .flatten()
                                .filter_map(|v| v.as_str().map(str::to_string))
                                .collect(),
                            env,
                            timeout_s: 60,
                            trust_hints: false,
                        },
                    );
                }
            }
            self.host
                .create_with_mcp(
                    OpEnvelope {
                        v: PROTOCOL,
                        session: session_id,
                        op_id: OpId::new(),
                        op: Op::StartSession {
                            cwd: cwd.clone(),
                            profile: self.config.meta.profile.clone(),
                            fork: None,
                        },
                    },
                    overrides,
                )
                .await
                .map(|(gateway, _)| gateway)
        };
        let gateway = match result {
            Ok(g) => g.for_face("acp"),
            Err(reason) => {
                self.error(
                    id,
                    -32002,
                    "Session unavailable",
                    Some(json!({"reason":reason})),
                )
                .await;
                return;
            }
        };
        if gateway.hello().meta.cwd != cwd {
            let _ = gateway.send(OpEnvelope {
                v: PROTOCOL,
                session: session_id,
                op_id: OpId::new(),
                op: Op::Close,
            });
            self.error(id, -32602, "Session cwd differs", None).await;
            return;
        }
        let mut subscription = match gateway.subscribe(Subscribe {
            session: session_id,
            after: None,
            ephemeral: !load,
        }) {
            Ok(s) => s,
            Err(_) => {
                self.error(id, -32002, "Session unavailable", None).await;
                return;
            }
        };
        subscription.set_approver(true);
        let blobs = Blobs {
            dir: self
                .config
                .sessions
                .join(session_id.to_string())
                .join("blobs"),
            redactor: self.config.redactor.clone(),
        };
        let session = Arc::new(Session {
            gateway,
            pending: Arc::default(),
            blobs,
        });
        self.sessions
            .lock()
            .unwrap()
            .insert(session_id, session.clone());
        let mut mapper = Mapper::default();
        let cutoff = subscription.hello().head;
        let live = if load {
            let mut live = session
                .gateway
                .subscribe(Subscribe {
                    session: session_id,
                    after: Some((subscription.hello().epoch, cutoff)),
                    ephemeral: true,
                })
                .expect("loaded session remains known");
            live.set_approver(true);
            Some(live)
        } else {
            None
        };
        // Register before replay, and finish replay before replying to session/load.
        loop {
            if self.cancel.is_cancelled() {
                return;
            }
            match subscription.try_recv() {
                TryRecv::Envelope(e) => {
                    let done = e.seq == Some(cutoff);
                    self.event(&session, &mut mapper, e, load).await;
                    if done {
                        break;
                    }
                }
                TryRecv::Empty | TryRecv::Closed => break,
            }
        }
        self.response(
            id,
            if load {
                json!({})
            } else {
                json!({"sessionId":session_id.to_string()})
            },
        )
        .await;
        if let Some(live) = live {
            subscription = live;
        }
        let face = self.clone();
        self.spawn(async move {
            face.observe(session, subscription, mapper).await;
        });
    }
    async fn observe(
        self: Arc<Self>,
        session: Arc<Session>,
        mut subscription: crate::gateway::Subscription,
        mut mapper: Mapper,
    ) {
        let session_id = session.gateway.session();
        loop {
            let event =
                tokio::select! {biased;_=self.cancel.cancelled()=>break,e=subscription.recv()=>e};
            let Some(event) = event else {
                break;
            };
            if let Event::Lagged { resume_after } = event.event {
                if let Ok(mut next) = session.gateway.subscribe(Subscribe {
                    session: session_id,
                    after: Some((event.epoch, resume_after)),
                    ephemeral: true,
                }) {
                    next.set_approver(true);
                    subscription = next;
                    continue;
                } else {
                    break;
                }
            }

            let closed = matches!(event.event, Event::SessionClosed);
            self.event(&session, &mut mapper, event, false).await;
            if closed {
                break;
            }
        }
    }
    async fn input(&self, session: &Session, blocks: &Value) -> Result<Vec<InputBlock>, String> {
        let mut out = vec![];
        for block in blocks.as_array().ok_or("Prompt is not an array")? {
            match block["type"].as_str() {
                Some("text") => out.push(InputBlock::Text {
                    text: block["text"].as_str().unwrap_or("").into(),
                }),
                Some("image") => {
                    use base64::Engine;
                    let bytes = base64::engine::general_purpose::STANDARD
                        .decode(block["data"].as_str().unwrap_or(""))
                        .map_err(|_| "Invalid image data")?;
                    let blob = session
                        .blobs
                        .put(
                            bytes,
                            block["mimeType"]
                                .as_str()
                                .unwrap_or("application/octet-stream"),
                        )
                        .await
                        .map_err(|_| "Image store failed")?;
                    out.push(InputBlock::Image { blob });
                }
                Some("resource_link") => {
                    let uri = block["uri"].as_str().unwrap_or("");
                    if let Ok(url) = reqwest::Url::parse(uri)
                        && url.scheme() == "file"
                        && let Ok(path) = url.to_file_path()
                    {
                        out.push(InputBlock::Mention { path });
                    } else {
                        out.push(InputBlock::Text {
                            text: format!("[unsupported resource link: {uri}]"),
                        });
                        session
                            .gateway
                            .notice(Level::Warn, "ACP resource link is not a file URI".into())
                            .await;
                    }
                }
                Some("resource") => {
                    let r = &block["resource"];
                    out.push(InputBlock::Text {
                        text: format!(
                            "{}:\n{}",
                            r["uri"].as_str().unwrap_or(""),
                            r["text"].as_str().unwrap_or("[binary resource]")
                        ),
                    });
                }
                _ => {
                    out.push(InputBlock::Text {
                        text: format!(
                            "[unsupported {} omitted]",
                            block["type"].as_str().unwrap_or("unknown")
                        ),
                    });
                    session
                        .gateway
                        .notice(Level::Warn, "ACP input content is unsupported".into())
                        .await;
                }
            }
        }
        if out.is_empty() {
            return Err("Prompt is empty".into());
        }
        Ok(out)
    }
    fn disconnected(&self) {
        let pending = std::mem::take(&mut *self.permissions.lock().unwrap());
        for (_, permission) in pending {
            let session = self
                .sessions
                .lock()
                .unwrap()
                .get(&permission.session)
                .cloned();
            if let Some(session) = session {
                let decision = match session.gateway.unattended() {
                    Unattended::Allow => ApprovalDecision::Allow,
                    Unattended::Deny => ApprovalDecision::Deny {
                        reason: Some("client disconnected".into()),
                    },
                };
                let _ = self.send(
                    &session,
                    Op::ResolveApproval {
                        approval: permission.item,
                        decision,
                    },
                );
            }
        }
    }
    async fn permission_response(&self, response: Value) {
        let Some(key) = response["id"].as_str() else {
            return;
        };
        let permission = self.permissions.lock().unwrap().remove(key);
        let Some(permission) = permission else {
            return;
        };
        let session = self
            .sessions
            .lock()
            .unwrap()
            .get(&permission.session)
            .cloned();
        let Some(session) = session else {
            return;
        };
        let outcome = &response["result"]["outcome"];
        let decision = if response.get("error").is_some() {
            match session.gateway.unattended() {
                Unattended::Allow => ApprovalDecision::Allow,
                Unattended::Deny => ApprovalDecision::Deny {
                    reason: Some("client unavailable".into()),
                },
            }
        } else if outcome["outcome"] == "selected" {
            match outcome["optionId"].as_str() {
                Some("allow") if permission.options.contains(&ApprovalDecisionKind::Allow) => {
                    ApprovalDecision::Allow
                }
                Some("allow_always")
                    if permission
                        .options
                        .contains(&ApprovalDecisionKind::AllowAlways) =>
                {
                    ApprovalDecision::AllowAlways
                }
                _ => ApprovalDecision::Deny {
                    reason: Some("client denied".into()),
                },
            }
        } else {
            ApprovalDecision::Deny {
                reason: Some("client cancelled".into()),
            }
        };
        let _ = self.send(
            &session,
            Op::ResolveApproval {
                approval: permission.item,
                decision,
            },
        );
    }
    async fn event(&self, session: &Session, mapper: &mut Mapper, e: Envelope, replay: bool) {
        let sid = e.session;
        match &e.event {
            Event::TurnStarted {
                origin: Some(TurnOrigin::Agent),
                ..
            } => {
                self.update(
                    sid,
                    chunk("agent_message_chunk", "agent continued\n".into()),
                )
                .await;
            }
            Event::TurnQueued { turn } => {
                if let Some(op) = e.cause {
                    let mut p = session.pending.lock().unwrap();
                    if let Some(id) = p.ops.remove(&op) {
                        p.turns.insert(*turn, id);
                    }
                }
            }
            Event::OpRejected { reason, .. } => {
                let id = e
                    .cause
                    .and_then(|op| session.pending.lock().unwrap().ops.remove(&op));
                if let Some(id) = id {
                    self.error(id, -32603, "Operation rejected", Some(json!(reason)))
                        .await;
                }
            }
            Event::TurnEnded { turn, reason, .. } => {
                let id = session.pending.lock().unwrap().turns.remove(turn);
                if let Some(id) = id {
                    match reason {
                        StopReason::Error {class,message}=>self.error(id,-32603,message,Some(json!({"class":class,"message":message}))).await,
                        _=>self.response(id,json!({"stopReason":match reason {StopReason::EndTurn=>"end_turn",StopReason::MaxTokens=>"max_tokens",StopReason::MaxRounds=>"max_turn_requests",_=>"cancelled"}})).await,
                    }
                }
            }
            _ => {}
        }
        let mut changed = None;
        let mut completed = false;
        match &e.event {
            Event::ContextChanged {
                mode,
                permission_mode,
                config,
            } => {
                self.update(
                    sid,
                    chunk(
                        "agent_message_chunk",
                        format!(
                            "Context: {} · {} · {} settings",
                            mode.as_deref().unwrap_or("unreported"),
                            permission_mode.as_deref().unwrap_or("unreported"),
                            config.as_ref().map_or(0, |c| c.len())
                        ),
                    ),
                )
                .await;
            }
            Event::ItemStarted {
                item,
                kind,
                parent,
                head,
                ..
            } => {
                let body = Item::interrupted_from_head(kind.clone(), *item, head.as_ref());
                mapper.items.insert(*item, (*parent, body));
                changed = Some(*item);
                if let Some(ItemHead::ToolCall {
                    name, tool_kind, ..
                }) = head
                {
                    self.update(sid,json!({"sessionUpdate":"tool_call","toolCallId":item.to_string(),"title":name,"kind":kind_name(tool_kind.clone()),"status":"in_progress"})).await;
                } else if let Some(ItemHead::Terminal { command, .. }) = head
                    && parent.is_none()
                {
                    self.update(sid,json!({"sessionUpdate":"tool_call","toolCallId":item.to_string(),"title":command,"kind":"execute","status":"in_progress"})).await;
                }
            }
            Event::ItemDelta { item, delta } => {
                if let Some((_, body)) = mapper.items.get_mut(item) {
                    body.apply(delta);
                    changed = Some(*item);
                }
            }
            Event::ItemSnapshot {
                item,
                partial: body,
                ..
            }
            | Event::ItemCompleted { item, body, .. } => {
                if let Some((_, stored)) = mapper.items.get_mut(item) {
                    *stored = body.clone();
                } else {
                    mapper.items.insert(*item, (None, body.clone()));
                }
                changed = Some(*item);
                completed = matches!(e.event, Event::ItemCompleted { .. });
            }
            _ => {}
        }
        let Some(item) = changed else {
            return;
        };
        let Some((parent, body)) = mapper.items.get(&item).cloned() else {
            return;
        };
        match body {
            Item::AgentMessage {markdown}=>if let Some(text)=mapper.suffix(item,&markdown){self.update(sid,chunk("agent_message_chunk",text)).await;},
            Item::Reasoning {summary,..}=>if let Some(text)=mapper.suffix(item,&summary){self.update(sid,chunk("agent_thought_chunk",text)).await;},
            Item::UserMessage {blocks} if completed&&replay=>self.update(sid,chunk("user_message_chunk",crate::engine::history::input(&blocks).as_str().unwrap_or("").into())).await,
            Item::ToolCall {input_raw,output,locations,title,server,raw_output,..}=> {
                let mut update=json!({"sessionUpdate":"tool_call_update","toolCallId":item.to_string()});
                if let Ok(input)=serde_json::from_str::<Value>(&input_raw) {update["rawInput"]=input;}
                if let Some(title)=title {update["title"]=json!(title);}
                if let Some(server)=server {update["_meta"]=json!({"bboxy450-harness":{"server":server}});}
                if let Some(raw)=raw_output {update["rawOutput"]=raw;}
                if completed {update["status"]=json!(if matches!(e.event,Event::ItemCompleted {status:ItemStatus::Completed,..}){"completed"}else{"failed"});update["content"]=mapper.content(item,item,content(clip(&crate::engine::history::output_text(&output),64*1024)));update["locations"]=json!(locations.iter().map(|l|{let mut v=json!({"path":l.path});if let Some(line)=l.line{v["line"]=json!(line);}v}).collect::<Vec<_>>());}
                if update.as_object().unwrap().len()>2 {self.update(sid,update).await;}
            }
            Item::FileChange {unified,..} if completed=> {if let Some(parent)=parent {let merged=mapper.content(parent,item,content(format!("```diff\n{}\n```",clip(&unified,64*1024-16))));self.update(sid,json!({"sessionUpdate":"tool_call_update","toolCallId":parent.to_string(),"content":merged})).await;}}
            Item::Terminal {command,exit,live,output,..}=> {
                let mut text=live.unwrap_or_default();
                if completed && let Some(blob)=output {text=session.blobs.get(&blob).await.map(|b|String::from_utf8_lossy(&b).into_owned()).unwrap_or_default();}
                let command=clip(&command,4096);
                let text=format!("$ {command}\n{}{}",clip(&text,64*1024-command.len()-64),exit.map(|n|format!("\nexit {n}")).unwrap_or_default());
                let target=parent.unwrap_or(item);let merged=mapper.content(target,item,content(text));
                let mut update=json!({"sessionUpdate":"tool_call_update","toolCallId":target.to_string(),"content":merged});
                if completed && parent.is_none(){update["status"]=json!(if matches!(e.event,Event::ItemCompleted {status:ItemStatus::Completed,..}){"completed"}else{"failed"});}
                if !matches!(e.event,Event::ItemStarted {..}){self.update(sid,update).await;}
            }
            Item::Plan {entries}=>self.update(sid,json!({"sessionUpdate":"plan","entries":entries.iter().map(|entry|json!({"content":if entry.status==PlanStatus::Cancelled {format!("(cancelled) {}",entry.text)}else{entry.text.clone()},"priority":entry.priority.clone().filter(|p|!matches!(p,Priority::Unknown(_))).unwrap_or(Priority::Medium),"status":match entry.status {PlanStatus::InProgress=>"in_progress",PlanStatus::Completed|PlanStatus::Cancelled=>"completed",_=>"pending"}})).collect::<Vec<_>>()})).await,
            Item::Approval {subject,options,decision,..} if !completed && decision.is_none() && !options.is_empty()=> {
                let id={let mut permissions=self.permissions.lock().unwrap();
                if permissions.values().any(|p|p.session==sid&&p.item==item){return;}
                let id=format!("permission-{}",self.next.fetch_add(1,Ordering::Relaxed));
                permissions.insert(id.clone(),Permission {session:sid,item,options:options.clone()});id};
                self.emit(json!({"jsonrpc":"2.0","id":id,"method":"session/request_permission","params":{"sessionId":sid.to_string(),"toolCall":{"toolCallId":subject.tool_call.to_string(),"title":subject.summary,"status":"pending"},"options":options.iter().filter_map(|option|match option {ApprovalDecisionKind::Allow=>Some(json!({"optionId":"allow","name":"Allow","kind":"allow_once"})),ApprovalDecisionKind::AllowAlways=>Some(json!({"optionId":"allow_always","name":"Always allow","kind":"allow_always"})),ApprovalDecisionKind::Deny=>Some(json!({"optionId":"deny","name":"Deny","kind":"reject_once"})),_=>None}).collect::<Vec<_>>()}})).await;
            }
            Item::Approval {..} if completed=>self.permissions.lock().unwrap().retain(|_,p|p.session!=sid||p.item!=item),
            Item::MemoryWrite { .. } | Item::MemoryState { .. } | Item::Trial { .. } | Item::Adoption { .. } | Item::Recall { .. } | Item::Command { .. } | Item::Hook { .. } | Item::SystemMessage { .. } | Item::Native { .. } if completed => self.update(sid,chunk("agent_message_chunk",body.extension_text().unwrap_or_default())).await,
            Item::Usage {usage} if completed && usage.context_used.is_some() && usage.context_window.is_some() => self.update(sid,json!({"sessionUpdate":"usage_update","used":usage.context_used,"size":usage.context_window,"_meta":{"bboxy450-harness":usage}})).await,
            Item::Usage {usage} if completed && usage.scope.is_some() => self.update(sid,chunk("agent_message_chunk",format!("Usage: {}", usage.numbers().into_iter().map(|(k,v)|format!("{k}={v}")).collect::<Vec<_>>().join(" · ")))).await,
            Item::Subagent {session,name,..} if completed && name.is_some() => self.update(sid,chunk("agent_message_chunk",format!("Subagent {} · {session}",name.as_deref().unwrap_or("")))).await,
            Item::Unknown(raw) if completed => self.update(sid,chunk("agent_message_chunk",format!("Unsupported item: {}",unknown_tag(&raw)))).await,
            Item::Notice {level:Level::Error,text}=>eprintln!("{}",self.config.redactor.scrub(&text)),
            _=>{},
        }
        if completed {
            mapper.items.remove(&item);
            mapper.sent.remove(&item);
            mapper.contents.remove(&item);
        }
    }
}
#[derive(Default)]
struct Mapper {
    items: BTreeMap<ItemId, (Option<ItemId>, Item)>,
    sent: BTreeMap<ItemId, String>,
    divergence: u64,
    contents: BTreeMap<ItemId, Vec<(ItemId, Value)>>,
}
impl Mapper {
    /// ACP updates replace content. Retain every child segment in arrival order;
    /// snapshots replace a segment, and parent completion adds its own result.
    fn content(&mut self, parent: ItemId, item: ItemId, content: Value) -> Value {
        let segments = self.contents.entry(parent).or_default();
        if let Some((_, stored)) = segments.iter_mut().find(|(id, _)| *id == item) {
            *stored = content;
        } else {
            segments.push((item, content));
        }
        Value::Array(
            segments
                .iter()
                .flat_map(|(_, value)| value.as_array().into_iter().flatten().cloned())
                .collect(),
        )
    }
    fn suffix(&mut self, item: ItemId, text: &str) -> Option<String> {
        let previous = self.sent.entry(item).or_default();
        if !text.starts_with(previous.as_str()) {
            self.divergence += 1;
            return None;
        }
        let suffix = text[previous.len()..].to_owned();
        *previous = text.into();
        (!suffix.is_empty()).then_some(suffix)
    }
}
fn kind_name(kind: ToolKind) -> &'static str {
    match kind {
        ToolKind::Read => "read",
        ToolKind::Edit => "edit",
        ToolKind::Delete => "delete",
        ToolKind::Move => "move",
        ToolKind::Search => "search",
        ToolKind::Execute => "execute",
        ToolKind::Think => "think",
        ToolKind::Fetch => "fetch",
        _ => "other",
    }
}
fn chunk(kind: &str, text: String) -> Value {
    json!({"sessionUpdate":kind,"content":{"type":"text","text":text}})
}
fn content(text: String) -> Value {
    json!([{"type":"content","content":{"type":"text","text":text}}])
}
fn clip(text: &str, limit: usize) -> String {
    if text.len() <= limit {
        return text.into();
    }
    if limit < 3 {
        return String::new();
    }
    let mut end = limit.saturating_sub(3);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}…", &text[..end])
}
fn bound(mut value: Value) -> Option<Value> {
    fn clip_fields(value: &mut Value, n: usize) {
        match value {
            Value::String(text) => *text = clip(text, n),
            Value::Array(values) => {
                for value in values {
                    clip_fields(value, n)
                }
            }
            Value::Object(values) => {
                for (key, value) in values {
                    if ["text", "content", "title", "message", "summary"].contains(&key.as_str())
                        || value.is_object()
                        || value.is_array()
                    {
                        clip_fields(value, n)
                    }
                }
            }
            _ => {}
        }
    }
    let mut limit = 64 * 1024;
    while serde_json::to_vec(&value).unwrap().len() > 256 * 1024 && limit > 0 {
        limit /= 2;
        clip_fields(&mut value, limit);
    }
    (serde_json::to_vec(&value).unwrap().len() <= 256 * 1024).then_some(value)
}
/// The caller owns this future; it returns only after every actor and driver joins.
pub async fn serve<
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
>(
    input: R,
    output: W,
    config: Config,
) -> std::io::Result<()> {
    serve_cancelled(input, output, config, CancellationToken::new()).await
}
pub async fn serve_cancelled<
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
>(
    input: R,
    output: W,
    config: Config,
    cancel: CancellationToken,
) -> std::io::Result<()> {
    let host = Host::new(config.clone());
    serve_with_host(input, output, config, host, cancel).await
}
pub(crate) async fn serve_with_host<
    R: AsyncRead + Unpin + Send + 'static,
    W: AsyncWrite + Unpin + Send + 'static,
>(
    input: R,
    mut output: W,
    config: Config,
    host: Host,
    cancel: CancellationToken,
) -> std::io::Result<()> {
    let (tx, mut rx) = mpsc::channel::<Value>(256);
    let (connection, mut requests) = mpsc::unbounded_channel();
    let face = Arc::new(Face {
        host,
        config,
        sessions: Mutex::default(),
        permissions: Mutex::default(),
        output: tx,
        drivers: Mutex::new(TaskGroup::new(cancel.clone())),
        inputs: Mutex::default(),
        connection,
        cancel: cancel.clone(),
        next: 0.into(),
        unknown: 0.into(),
    });
    let handler = face.clone();
    face.spawn(async move {
        loop {
            let request =
                tokio::select! {biased;_=handler.cancel.cancelled()=>break,r=requests.recv()=>r};
            match request {
                Some(ConnectionInput::Message(request)) => handler.clone().dispatch(request).await,
                Some(ConnectionInput::ParseError) => {
                    handler
                        .error(Value::Null, -32700, "Parse error", None)
                        .await
                }
                None => break,
            }
        }
    });
    let mut tasks = TaskGroup::new(cancel.clone());
    let stop = cancel.clone();
    tasks.spawn(async move {
        loop {
            let value = tokio::select! {biased;_=stop.cancelled()=>break,v=rx.recv()=>v};
            let Some(value) = value else {
                break;
            };
            let mut bytes = serde_json::to_vec(&value).unwrap();
            bytes.push(b'\n');
            let result =
                tokio::select! {biased;_=stop.cancelled()=>break,r=output.write_all(&bytes)=>r};
            if result.is_err() {
                stop.cancel();
                break;
            }
        }
    });
    let mut lines = BufReader::new(input).lines();
    loop {
        let line = tokio::select! {biased;_=cancel.cancelled()=>break,line=lines.next_line()=>line};
        match line {
            Ok(Some(line)) => match serde_json::from_str::<Value>(&line) {
                Ok(request) => face.route(request),
                Err(_) => {
                    let _ = face.connection.send(ConnectionInput::ParseError);
                }
            },
            _ => break,
        }
        tasks.reap().await;
    }
    face.disconnected();
    cancel.cancel();
    face.host.close_all().await;
    let mut drivers = std::mem::replace(
        &mut *face.drivers.lock().unwrap(),
        TaskGroup::new(cancel.clone()),
    );
    let _ = drivers.cancel_and_join().await;
    let _ = tasks.cancel_and_join().await;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn admission_lag_replays_the_durable_prompt_before_next_input() {
        let meta = SessionMeta {
            vcs: None,
            dream: None,
            source: None,
            session: SessionId::new(),
            cwd: std::env::temp_dir(),
            profile: None,
            fork: None,
            protocol: PROTOCOL,
            harness: "test".into(),
            created_ms: 0,
        };
        let session = meta.session;
        let start = Envelope {
            raw: None,
            ext: Default::default(),
            native: None,
            v: PROTOCOL,
            session,
            epoch: Epoch(1),
            seq: Some(Seq(1)),
            at: Seq(1),
            ts_ms: 0,
            cause: None,
            event: Event::SessionStarted { meta: meta.clone() },
        };
        let hub = Arc::new(crate::gateway::Hub::new(
            meta,
            Epoch(1),
            std::slice::from_ref(&start),
            crate::redact::Set::default(),
        ));
        let (work, _work) = mpsc::channel(1);
        let (control, _control) = mpsc::channel(1);
        let gateway = InProc::new(hub.clone(), work, control, OpId::new());
        let mut sub = gateway
            .subscribe(Subscribe {
                session,
                after: Some((Epoch(1), Seq(1))),
                ephemeral: false,
            })
            .unwrap();
        let mut log = vec![start];
        let mut seq = 1;
        let mut publish = |event, cause| {
            seq += 1;
            let event = Envelope {
                raw: None,
                ext: Default::default(),
                native: None,
                v: PROTOCOL,
                session,
                epoch: Epoch(1),
                seq: Some(Seq(seq)),
                at: Seq(seq),
                ts_ms: 0,
                cause,
                event,
            };
            hub.publish(event.clone());
            log.push(event);
        };
        // Overflow the actual 4096-slot gateway queue before the prompt's
        // durable outcome. Replay must recover that outcome from the real hub.
        for _ in 0..2050 {
            let item = ItemId::new();
            publish(
                Event::ItemStarted {
                    turn: None,
                    item,
                    kind: ItemKind::Notice,
                    parent: None,
                    head: None,
                },
                None,
            );
            publish(
                Event::ItemCompleted {
                    item,
                    status: ItemStatus::Completed,
                    body: Item::Notice {
                        level: Level::Info,
                        text: "prior event".into(),
                    },
                },
                None,
            );
        }
        let op_id = OpId::new();
        let turn = TurnId::new();
        publish(Event::TurnQueued { turn }, Some(op_id));
        publish(
            Event::TurnEnded {
                turn,
                reason: StopReason::Cancelled,
                usage: None,
            },
            None,
        );
        publish(Event::SessionClosed, None);
        let lagged = Arc::new(AtomicU64::new(0));
        let seen = lagged.clone();
        let held_eof = Box::pin(async_stream::stream! {
            while let Some(event)=sub.recv().await {
                let is_lagged=matches!(event.event,Event::Lagged {..});
                if is_lagged {seen.fetch_add(1,Ordering::Relaxed);}
                yield event;
                // InProc currently closes after Lagged. Hold that EOF to prove
                // recovery follows Lagged itself, without relying on closure.
                if is_lagged {std::future::pending::<()>().await;}
            }
        });
        let cancel = CancellationToken::new();
        let admitted = tokio::time::timeout(
            std::time::Duration::from_secs(10),
            await_prompt_admission(&gateway, op_id, held_eof, &cancel),
        )
        .await
        .expect("admission-lag hung without replay")
        .expect("prompt outcome missing after lag");
        assert_eq!(
            lagged.load(Ordering::Relaxed),
            1,
            "real gateway did not lag"
        );
        assert_eq!(admitted.cause, Some(op_id));
        assert!(matches!(admitted.event,Event::TurnQueued {turn:t} if t==turn));
        crate::protocol::laws::check_transcript(&log).unwrap();
    }
    #[tokio::test]
    async fn agent_turn_fixture_exports_its_heading_and_work_without_a_prompt() {
        let events: Vec<Envelope> =
            include_str!("../../tests/fixtures/ir-v1/v1.2-agent-turn.jsonl")
                .lines()
                .map(|l| serde_json::from_str(l).unwrap())
                .collect();
        let temp = tempfile::tempdir().unwrap();
        let mut meta = match &events[0].event {
            Event::SessionStarted { meta } => meta.clone(),
            _ => panic!("fixture metadata missing"),
        };
        meta.cwd = temp.path().into();
        meta.profile = None;
        let sid = meta.session;
        let config = Config {
            meta,
            sessions: temp.path().join("sessions"),
            profiles: temp.path().join("profiles"),
            provider: Arc::new(crate::provider::ScriptedProvider::new(vec![])),
            model: "scripted".into(),
            policy: crate::policy::Policy::new(Unattended::Deny),
            redactor: Default::default(),
            max_rounds: 1,
            profile_error: None,
        };
        let host = Host::new(config.clone());
        let gateway = host
            .create(OpEnvelope {
                v: PROTOCOL,
                session: sid,
                op_id: OpId::new(),
                op: Op::StartSession {
                    cwd: temp.path().into(),
                    profile: None,
                    fork: None,
                },
            })
            .await
            .unwrap()
            .0;
        let session = Session {
            gateway: gateway.clone(),
            pending: Arc::default(),
            blobs: Blobs {
                dir: config.sessions.join(sid.to_string()).join("blobs"),
                redactor: config.redactor.clone(),
            },
        };
        let (output, mut rx) = mpsc::channel(128);
        let (connection, _requests) = mpsc::unbounded_channel();
        let cancel = CancellationToken::new();
        let face = Face {
            host: Host::new(config.clone()),
            config,
            sessions: Mutex::default(),
            permissions: Mutex::default(),
            output,
            drivers: Mutex::new(TaskGroup::new(cancel.clone())),
            inputs: Mutex::default(),
            connection,
            cancel,
            next: 0.into(),
            unknown: 0.into(),
        };
        let mut mapper = Mapper::default();
        for event in events {
            face.event(&session, &mut mapper, event, true).await;
        }
        let mut text = String::new();
        let mut tools = 0;
        while let Ok(value) = rx.try_recv() {
            assert_eq!(
                value["method"], "session/update",
                "agent turn must not invent a prompt response"
            );
            serde_json::from_value::<schema::SessionNotification>(value["params"].clone()).unwrap();
            let update = &value["params"]["update"];
            if update["sessionUpdate"] == "agent_message_chunk" {
                text.push_str(update["content"]["text"].as_str().unwrap());
            }
            if update["sessionUpdate"] == "tool_call_update" {
                tools += 1;
            }
        }
        for expected in ["agent continued", "woke", "done"] {
            assert!(text.contains(expected), "{text}");
        }
        assert!(tools > 0);
        gateway
            .send(OpEnvelope {
                v: PROTOCOL,
                session: sid,
                op_id: OpId::new(),
                op: Op::Close,
            })
            .unwrap();
        host.join().await.unwrap();
    }
    struct HeldFinish {
        inner: crate::provider::ScriptedProvider,
        release: Arc<tokio::sync::Notify>,
    }
    impl crate::provider::Provider for HeldFinish {
        fn caps(&self) -> crate::provider::Caps {
            self.inner.caps()
        }
        fn stream(
            &self,
            request: crate::provider::ModelRequest,
            cancel: CancellationToken,
        ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = crate::provider::ModelEvent> + Send>>
        {
            use futures_util::StreamExt;
            let mut inner = self.inner.stream(request, cancel.clone());
            let release = self.release.clone();
            Box::pin(
                async_stream::stream! {while let Some(event)=inner.next().await {if matches!(event,crate::provider::ModelEvent::Finish(_)) {tokio::select!{biased;_=cancel.cancelled()=>return,_=release.notified()=>{}}}yield event;}},
            )
        }
    }
    #[tokio::test]
    async fn real_stream_with_forced_ephemeral_drops_repairs_from_completion_and_snapshot() {
        let temp = tempfile::tempdir().unwrap();
        let sid = SessionId::new();
        let text = "aλ🙂";
        let thought = "inspect λ";
        let chunk = json!({"choices":[{"delta":{"content":text,"reasoning":thought},"finish_reason":null}]});
        let end = json!({"choices":[{"delta":{},"finish_reason":"stop"}]});
        let release = Arc::new(tokio::sync::Notify::new());
        let provider = crate::provider::ScriptedProvider::new(vec![
            format!("data: {chunk}\n\ndata: {end}\n\ndata: [DONE]\n\n").into_bytes(),
        ]);
        let provider = HeldFinish {
            inner: provider,
            release: release.clone(),
        };
        let config = Config {
            meta: SessionMeta {
                vcs: None,
                dream: None,
                source: None,
                session: sid,
                cwd: temp.path().into(),
                profile: None,
                fork: None,
                protocol: PROTOCOL,
                harness: "test".into(),
                created_ms: 0,
            },
            sessions: temp.path().join("sessions"),
            profiles: temp.path().join("profiles"),
            provider: Arc::new(provider),
            model: "mock".into(),
            policy: crate::policy::Policy::new(Unattended::Deny),
            redactor: crate::redact::Set::default(),
            max_rounds: 2,
            profile_error: None,
        };
        let host = Host::new(config.clone());
        let gateway = host
            .create(OpEnvelope {
                v: PROTOCOL,
                session: sid,
                op_id: OpId::new(),
                op: Op::StartSession {
                    cwd: temp.path().into(),
                    profile: None,
                    fork: None,
                },
            })
            .await
            .unwrap()
            .0;
        let mut sub = gateway
            .subscribe(Subscribe {
                session: sid,
                after: None,
                ephemeral: true,
            })
            .unwrap();
        gateway
            .send(OpEnvelope {
                v: PROTOCOL,
                session: sid,
                op_id: OpId::new(),
                op: Op::UserTurn {
                    blocks: vec![InputBlock::Text {
                        text: "hello".into(),
                    }],
                    mode: TurnMode::Queue,
                },
            })
            .unwrap();
        let mut events = vec![];
        loop {
            let e = tokio::time::timeout(std::time::Duration::from_secs(8), sub.recv())
                .await
                .expect("stream never produced its retained snapshot")
                .unwrap();
            if matches!(e.event, Event::ItemSnapshot { .. }) {
                release.notify_one();
            }
            let done = matches!(e.event, Event::TurnEnded { .. });
            events.push(e);
            if done {
                break;
            }
        }
        let blobs = Blobs {
            dir: config.sessions.join(sid.to_string()).join("blobs"),
            redactor: config.redactor.clone(),
        };
        let session = Session {
            gateway: gateway.clone(),
            pending: Arc::default(),
            blobs,
        };
        let mut runs = vec![];
        for _ in 0..3 {
            let (output, rx) = mpsc::channel(64);
            let (connection, _requests) = mpsc::unbounded_channel();
            let cancel = CancellationToken::new();
            runs.push((
                Face {
                    host: Host::new(config.clone()),
                    config: config.clone(),
                    sessions: Mutex::default(),
                    permissions: Mutex::default(),
                    output,
                    drivers: Mutex::new(TaskGroup::new(cancel.clone())),
                    inputs: Mutex::default(),
                    connection,
                    cancel,
                    next: 0.into(),
                    unknown: 0.into(),
                },
                Mapper::default(),
                rx,
            ));
        }
        let mut drops = 0;
        for event in &events {
            for (mode, (face, mapper, _)) in runs.iter_mut().enumerate() {
                if mode > 0
                    && matches!(
                        event.event,
                        Event::ItemDelta { .. } | Event::ItemSnapshot { .. }
                    )
                {
                    drops += 1;
                    continue;
                }
                if mode == 2
                    && let Event::ItemCompleted { item, body, .. } = &event.event
                {
                    let mut snapshot = event.clone();
                    snapshot.seq = None;
                    snapshot.at = Seq(event.at.0 - 1);
                    snapshot.event = Event::ItemSnapshot {
                        item: *item,
                        partial: body.clone(),
                    };
                    face.event(&session, mapper, snapshot, false).await;
                }
                face.event(&session, mapper, event.clone(), false).await;
            }
        }
        gateway
            .send(OpEnvelope {
                v: PROTOCOL,
                session: sid,
                op_id: OpId::new(),
                op: Op::Close,
            })
            .unwrap();
        host.join().await.unwrap();
        crate::protocol::laws::check_transcript(&events).unwrap();
        assert!(
            drops > 0,
            "no ephemeral events were dropped: {:?}",
            events.iter().map(|e| &e.event).collect::<Vec<_>>()
        );
        let mut rendered = vec![];
        for (_, mapper, mut rx) in runs {
            assert_eq!(mapper.divergence, 0);
            let mut chunks = BTreeMap::<String, String>::new();
            while let Ok(value) = rx.try_recv() {
                let update = &value["params"]["update"];
                if let Some(kind) = update["sessionUpdate"]
                    .as_str()
                    .filter(|k| k.ends_with("_chunk"))
                {
                    chunks
                        .entry(kind.into())
                        .or_default()
                        .push_str(update["content"]["text"].as_str().unwrap());
                }
            }
            rendered.push(chunks);
        }
        assert_eq!(rendered[0], rendered[1]);
        assert_eq!(rendered[0], rendered[2]);
        assert_eq!(rendered[0]["agent_message_chunk"], text);
        assert_eq!(rendered[0]["agent_thought_chunk"], thought);
    }
}
