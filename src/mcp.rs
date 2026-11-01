// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Session-owned MCP stdio services. SDK drivers, pipes and process groups join at close.
use crate::{
    engine::{
        Internal,
        task_group::{self, TaskGroup},
    },
    policy::EffectClass,
    protocol::*,
    provider::ToolDef,
    redact::Set,
    tools::{Concurrency, Definition, ToolOutput},
};
use arc_swap::ArcSwap;
use rmcp::{
    ClientHandler, Peer, RoleClient, ServiceExt,
    model::*,
    service::{NotificationContext, PeerRequestOptions, RxJsonRpcMessage, TxJsonRpcMessage},
    transport::{TokioChildProcess, Transport},
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::AsyncReadExt,
    sync::{Notify, mpsc, oneshot},
};
use tokio_util::sync::CancellationToken;
#[derive(Clone, Deserialize, Serialize)]
pub struct ServerConfig {
    pub command: String,
    #[serde(default)]
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default = "default_timeout")]
    pub timeout_s: u64,
    #[serde(default)]
    pub trust_hints: bool,
}
fn default_timeout() -> u64 {
    60
}
impl std::fmt::Debug for ServerConfig {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ServerConfig")
            .field("command", &"<redacted>")
            .field("env", &"<redacted>")
            .finish()
    }
}
pub type Servers = BTreeMap<String, ServerConfig>;
#[derive(Clone, Copy, PartialEq, Eq)]
enum Origin {
    Profile,
    Acp,
    Launcher,
}
impl Origin {
    fn label(self) -> &'static str {
        match self {
            Self::Profile => "profile",
            Self::Acp => "ACP",
            Self::Launcher => "launcher",
        }
    }
}
pub(crate) struct SpawnConfig {
    config: ServerConfig,
    origin: Origin,
}
pub(crate) type SpawnServers = BTreeMap<String, SpawnConfig>;
/// Provenance is assigned by the host, never deserialized from server data.
pub(crate) fn select_sources(
    profile: Servers,
    acp: Servers,
    launcher: Servers,
) -> (SpawnServers, Vec<String>) {
    let mut selected = SpawnServers::new();
    let mut notices = BTreeMap::new();
    for (origin, servers) in [
        (Origin::Profile, profile),
        (Origin::Acp, acp),
        (Origin::Launcher, launcher),
    ] {
        for (name, config) in servers {
            if let Some(prior) = selected.insert(name.clone(), SpawnConfig { config, origin }) {
                notices.insert(
                    name.clone(),
                    format!(
                        "mcp {name}: {} configuration overrides {}",
                        origin.label(),
                        prior.origin.label()
                    ),
                );
            }
        }
    }
    (selected, notices.into_values().collect())
}
#[cfg(test)]
mod source_tests {
    use super::*;
    #[test]
    fn reserved_reference_is_a_typed_config_error_before_resolution() {
        for source in ["$BBOXY_SOCK", "${BBOXY_SEAT_ID}", "!echo ${BBOXY_ROLE}"] {
            let env = [("alias".into(), source.into())].into();
            for origin in [Origin::Profile, Origin::Acp] {
                let error = validate_environment(origin, &env).unwrap_err();
                assert_eq!(error.class, ErrorClass::Config);
                assert_eq!(error.message, "reserved environment reference");
            }
            assert!(validate_environment(Origin::Launcher, &env).is_ok());
        }
    }
    #[test]
    fn launcher_provenance_cannot_come_from_server_json_and_overrides_once() {
        let server = |command: &str| {
            serde_json::from_value::<ServerConfig>(json!({"command":command,"launcher":true}))
                .unwrap()
        };
        let (profile, _) = select_sources(
            [("bboxy".into(), server("profile"))].into(),
            Default::default(),
            Default::default(),
        );
        assert!(profile["bboxy"].origin == Origin::Profile);
        let (selected, notices) = select_sources(
            [("bboxy".into(), server("profile"))].into(),
            [("bboxy".into(), server("acp"))].into(),
            [("bboxy".into(), server("launcher"))].into(),
        );
        assert_eq!(selected["bboxy"].config.command, "launcher");
        assert!(selected["bboxy"].origin == Origin::Launcher);
        assert_eq!(
            notices,
            vec!["mcp bboxy: launcher configuration overrides ACP"]
        );
    }
}
#[derive(Clone)]
struct Ready {
    generation: u64,
    peer: Peer<RoleClient>,
    tools: Vec<Tool>,
}
struct Slot {
    name: String,
    config: ServerConfig,
    origin: Origin,
    ready: ArcSwap<Option<Ready>>,
    generation: std::sync::atomic::AtomicU64,
    state: std::sync::atomic::AtomicU8,
}
#[derive(Clone)]
pub struct Binding {
    slot: Arc<Slot>,
    generation: u64,
    tool: String,
    notices: mpsc::Sender<Internal>,
    root: CancellationToken,
}
#[derive(Clone)]
pub struct Manager {
    slots: Arc<Vec<Arc<Slot>>>,
    notices: mpsc::Sender<Internal>,
    root: CancellationToken,
}
impl Manager {
    pub(crate) fn start(
        config: SpawnServers,
        redactor: Set,
        output: mpsc::Sender<Internal>,
        services: &mut TaskGroup,
    ) -> Self {
        let root = services.cancel.clone();
        let mut slots = vec![];
        for (i, (name, selected)) in config.into_iter().enumerate() {
            if i >= 16 {
                let tx = output.clone();
                let cancel = root.clone();
                services.spawn(async move {
                    notice(
                        &tx,
                        &cancel,
                        Level::Warn,
                        format!("mcp {name}: server limit 16"),
                    )
                    .await;
                });
                continue;
            }
            let slot = Arc::new(Slot {
                name,
                config: selected.config,
                origin: selected.origin,
                ready: ArcSwap::from_pointee(None),
                generation: 0.into(),
                state: 0.into(),
            });
            let s = slot.clone();
            let tx = output.clone();
            let r = redactor.clone();
            let cancel = root.child_token();
            services.spawn(async move {
                supervise(s, r, tx, cancel).await;
            });
            slots.push(slot);
        }
        Self {
            slots: Arc::new(slots),
            notices: output,
            root,
        }
    }
    /// No await: the tool definitions and generation bindings are fixed at TurnStarted.
    pub(crate) fn snapshot(&self) -> (Vec<Definition>, BTreeMap<String, Binding>, Vec<String>) {
        let mut defs = vec![];
        let mut bindings = BTreeMap::new();
        let mut warnings = vec![];
        for slot in self.slots.iter() {
            let view = slot.ready.load();
            let Some(ready) = view.as_ref() else {
                continue;
            };
            let mut tools = ready.tools.clone();
            tools.sort_by(|a, b| a.name.cmp(&b.name));
            for tool in tools {
                let normalize = |s: &str| {
                    s.chars()
                        .map(|c| {
                            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                                c
                            } else {
                                '_'
                            }
                        })
                        .collect::<String>()
                };
                let name = format!("mcp__{}__{}", normalize(&slot.name), normalize(&tool.name))
                    .chars()
                    .take(64)
                    .collect::<String>();
                if bindings.contains_key(&name) {
                    warnings.push(format!("mcp {}: tool name collision {name}", slot.name));
                    continue;
                }
                if bindings.len() >= 128 {
                    if !warnings.iter().any(|s| s.contains("tool limit 128")) {
                        warnings.push(format!("mcp {}: tool limit 128", slot.name));
                    }
                    break;
                }
                let annotations = serde_json::to_value(&tool.annotations).unwrap_or(Value::Null);
                let read = annotations["readOnlyHint"].as_bool() == Some(true);
                let trusted = read && slot.config.trust_hints;
                defs.push(Definition {
                    wire: ToolDef {
                        name: name.clone(),
                        description: tool.description.as_deref().unwrap_or("").to_string(),
                        parameters: json!(tool.input_schema),
                    },
                    effect: if trusted {
                        EffectClass::ReadOnly
                    } else if annotations["openWorldHint"] == true {
                        EffectClass::Network
                    } else {
                        EffectClass::Exec
                    },
                    concurrency: if read {
                        Concurrency::ReadOnly
                    } else {
                        Concurrency::Exclusive
                    },
                    kind: if trusted {
                        ToolKind::Read
                    } else {
                        ToolKind::Other
                    },
                });
                bindings.insert(
                    name,
                    Binding {
                        slot: slot.clone(),
                        generation: ready.generation,
                        tool: tool.name.to_string(),
                        notices: self.notices.clone(),
                        root: self.root.clone(),
                    },
                );
            }
        }
        (defs, bindings, warnings)
    }
}
async fn notice(
    tx: &mpsc::Sender<Internal>,
    cancel: &CancellationToken,
    level: Level,
    text: String,
) {
    let _ = task_group::send(tx, Internal::ServiceNotice { level, text }, cancel).await;
}
impl Binding {
    pub async fn call(&self, args: Value, ctx: &crate::tools::Context) -> ToolOutput {
        use std::sync::atomic::Ordering;
        let view = self.slot.ready.load_full();
        let Some(ready) = view.as_ref().as_ref() else {
            let state = ["starting", "ready", "failed", "dead"]
                [usize::from(self.slot.state.load(Ordering::Acquire))];
            return ToolOutput::error(format!("mcp {}: {state}", self.slot.name));
        };
        if ready.generation != self.generation {
            return ToolOutput::error(format!("mcp {}: stale generation", self.slot.name));
        }
        let args = ctx.redactor.value(args);
        let mut params = CallToolRequestParams::new(self.tool.clone());
        params.arguments = args.as_object().cloned();
        let request = ClientRequest::CallToolRequest(CallToolRequest::new(params));
        let sent = tokio::select! {biased;_=ctx.cancel.cancelled()=>return ToolOutput::error("cancelled"),r=ready.peer.send_request_with_option(request,PeerRequestOptions::with_timeout(Duration::from_secs(self.slot.config.timeout_s)))=>r};
        let handle = match sent {
            Ok(h) => h,
            Err(_) => return ToolOutput::error(format!("mcp {}: server exited", self.slot.name)),
        };
        let id = handle.id.clone();
        let mut response = Box::pin(handle.await_response());
        let result = tokio::select! {biased;_=ctx.cancel.cancelled()=>{let _=tokio::time::timeout(Duration::from_millis(100),ready.peer.notify_cancelled(CancelledNotificationParam::new(Some(id),Some("turn cancelled".into())))).await;return ToolOutput::error("cancelled");},result=&mut response=>result};
        if ctx.current_generation.load(Ordering::Acquire) != ctx.generation
            || self.slot.generation.load(Ordering::Acquire) != self.generation
        {
            return ToolOutput::error(format!(
                "mcp {}: cancelled or stale generation",
                self.slot.name
            ));
        }
        let result = match result {
            Ok(ServerResult::CallToolResult(result)) => result,
            Err(rmcp::ServiceError::Timeout { .. }) => {
                let message = format!(
                    "mcp {}: timed out after {} s",
                    self.slot.name, self.slot.config.timeout_s
                );
                notice(&self.notices, &self.root, Level::Warn, message.clone()).await;
                return ToolOutput::error(message);
            }
            Err(rmcp::ServiceError::McpError(error)) => {
                let text = ctx
                    .redactor
                    .scrub(&format!("mcp {}: {}", self.slot.name, error.message));
                let mut output = ToolOutput::error(crate::context::clip_output(&text));
                if text.len() > 50 * 1024 {
                    output.spill = ctx.blobs.put(text.into_bytes(), "text/plain").await.ok();
                }
                return output;
            }
            _ => return ToolOutput::error(format!("mcp {}: server exited", self.slot.name)),
        };
        let value = ctx
            .redactor
            .value(serde_json::to_value(result).unwrap_or(Value::Null));
        let mut output = vec![];
        if let Some(content) = value["content"].as_array() {
            for block in content {
                match block["type"].as_str() {
                    Some("text") => output.push(OutputBlock::Text {
                        text: block["text"].as_str().unwrap_or("").into(),
                    }),
                    Some("image") => {
                        use base64::Engine;
                        if let Ok(bytes) = base64::engine::general_purpose::STANDARD
                            .decode(block["data"].as_str().unwrap_or(""))
                            && let Ok(blob) = ctx
                                .blobs
                                .put(
                                    bytes,
                                    block["mimeType"]
                                        .as_str()
                                        .unwrap_or("application/octet-stream"),
                                )
                                .await
                        {
                            output.push(OutputBlock::Image { blob });
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(value) = value.get("structuredContent") {
            output.push(OutputBlock::Text {
                text: serde_json::to_string_pretty(value).unwrap_or_default(),
            });
        }
        if output.is_empty() {
            output.push(OutputBlock::Text {
                text: String::new(),
            });
        }
        let text = crate::engine::history::output_text(&output);
        let spill = if text.len() > 50 * 1024 {
            ctx.blobs
                .put(text.as_bytes().to_vec(), "text/plain")
                .await
                .ok()
        } else {
            None
        };
        if spill.is_some() {
            let mut end = (50 * 1024).min(text.len());
            while !text.is_char_boundary(end) {
                end -= 1;
            }
            output = vec![OutputBlock::Text {
                text: format!("{}…", &text[..end]),
            }];
        }
        ToolOutput {
            output,
            spill,
            changes: vec![],
            terminal: None,
            pgid: None,
            failed: value["isError"] == true,
        }
    }
}
struct Handler {
    changed: Arc<Notify>,
}
impl ClientHandler for Handler {
    fn get_info(&self) -> ClientConfig {
        let mut info = ClientConfig::default();
        info.protocol_version = ProtocolVersion::LATEST_WITH_INITIALIZE;
        info
    }
    async fn on_tool_list_changed(&self, _: NotificationContext<RoleClient>) {
        self.changed.notify_one();
    }
}
type Child = Box<dyn process_wrap::tokio::ChildWrapper>;
/// Disarm the SDK's detached Drop cleanup. The session supervisor owns the child.
struct OwnedTransport {
    inner: Option<TokioChildProcess>,
    child: Option<oneshot::Sender<Child>>,
}
impl OwnedTransport {
    fn release(&mut self) {
        if let Some(inner) = self.inner.take()
            && let Some(child) = inner.into_inner()
            && let Some(tx) = self.child.take()
        {
            let _ = tx.send(child);
        }
    }
}
impl Drop for OwnedTransport {
    fn drop(&mut self) {
        self.release();
    }
}
impl Transport<RoleClient> for OwnedTransport {
    type Error = std::io::Error;
    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleClient>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        self.inner.as_mut().expect("open transport").send(item)
    }
    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleClient>> {
        self.inner.as_mut()?.receive().await
    }
    async fn close(&mut self) -> Result<(), Self::Error> {
        self.release();
        Ok(())
    }
}
async fn finish_child(rx: oneshot::Receiver<Child>, mut group: crate::process::OwnedGroup) {
    if let Ok(mut child) = rx.await {
        if child.id().is_some() {
            // The leader remains unreaped, reserving its pgid until both signals.
            group.send(libc::SIGTERM);
            tokio::time::sleep(Duration::from_secs(1)).await;
            group.send(libc::SIGKILL);
        }
        group.reaped();
        let _ = child.wait().await;
    }
}
fn scrub_bboxy_env(command: &mut tokio::process::Command) {
    let mut names = std::env::vars_os()
        .map(|(name, _)| name)
        .collect::<Vec<_>>();
    names.extend(command.as_std().get_envs().map(|(name, _)| name.to_owned()));
    for name in names {
        if name.as_encoded_bytes().starts_with(b"BBOXY_") {
            command.env_remove(name);
        }
    }
}
fn validate_environment(
    origin: Origin,
    env: &BTreeMap<String, String>,
) -> Result<(), crate::provider::ProviderError> {
    if origin != Origin::Launcher
        && env
            .values()
            .any(|value| value.contains("$BBOXY_") || value.contains("${BBOXY_"))
    {
        return Err(crate::provider::ProviderError::new(
            ErrorClass::Config,
            "reserved environment reference",
        ));
    }
    Ok(())
}
async fn supervise(
    slot: Arc<Slot>,
    redactor: Set,
    output: mpsc::Sender<Internal>,
    cancel: CancellationToken,
) {
    use std::sync::atomic::Ordering;
    // Invalid configuration is terminal for this slot, not a restartable server
    // failure. Validate all values before running even the first resolver.
    if let Err(error) = validate_environment(slot.origin, &slot.config.env) {
        slot.state.store(3, Ordering::Release);
        notice(
            &output,
            &cancel,
            Level::Warn,
            format!("mcp {}: configuration error: {}", slot.name, error.message),
        )
        .await;
        return;
    }
    for attempt in 0..5u32 {
        if cancel.is_cancelled() {
            return;
        }
        slot.state.store(0, Ordering::Release);
        notice(
            &output,
            &cancel,
            if attempt == 0 {
                Level::Info
            } else {
                Level::Warn
            },
            format!("mcp {}: starting", slot.name),
        )
        .await;
        let generation = slot.generation.fetch_add(1, Ordering::AcqRel) + 1;
        let stderr = Arc::new(Mutex::new(Vec::<u8>::new()));
        let result=async {
            let mut command=tokio::process::Command::new(&slot.config.command);
            command.args(&slot.config.args).process_group(0).kill_on_drop(true);
            if let Ok(cap)=std::env::var("BBOXY_CAPABILITY") {redactor.register(&cap);}
            for (key,source) in &slot.config.env {
                if slot.origin!=Origin::Launcher && key.starts_with("BBOXY_") {continue;}
                let value=if let Some(cmd)=source.strip_prefix('!') {crate::provider::profile::resolve_command(cmd,cancel.child_token()).await.map_err(|_|"environment command failed")?} else if let Some(name)=source.strip_prefix('$') {std::env::var(name).map_err(|_|"environment variable unavailable")?}else{source.clone()};
                if slot.origin!=Origin::Launcher && std::env::var("BBOXY_CAPABILITY").ok().is_some_and(|cap|!cap.is_empty()&&value.contains(&cap)) {return Err("reserved capability in environment");}
                redactor.register(&value);command.env(key,value);
            }
            if slot.origin!=Origin::Launcher {scrub_bboxy_env(&mut command);}
            let reservation=crate::process::reserve_group().map_err(|_|"process shutting down")?;
            let (transport,err)=TokioChildProcess::builder(command).stderr(std::process::Stdio::piped()).spawn().map_err(|_|"spawn failed")?;
            let group=crate::process::OwnedGroup::new(transport.id().expect("spawned MCP child"),reservation);
            let (tx,rx)=oneshot::channel();
            let mut pipes=TaskGroup::new(cancel.child_token());
            if let Some(mut err)=err {let ring=stderr.clone();let stop=pipes.cancel.clone();pipes.spawn(async move {let mut buf=[0u8;4096];loop {let n=tokio::select!{biased;_=stop.cancelled()=>break,n=err.read(&mut buf)=>n};let Ok(n)=n else {break;};if n==0{break;}let mut ring=ring.lock().unwrap();ring.extend_from_slice(&buf[..n]);let excess=ring.len().saturating_sub(64*1024);ring.drain(..excess);}});}
            let changed=Arc::new(Notify::new());
            let transport=OwnedTransport {inner:Some(transport),child:Some(tx)};
            let run=async {
                let deadline=tokio::time::Instant::now()+Duration::from_secs(10);
                let handshake=tokio::time::timeout_at(deadline,Handler {changed:changed.clone()}.serve(transport));
                let service=tokio::select! {biased;_=cancel.cancelled()=>return Err("cancelled"),r=handshake=>r.map_err(|_|"initialize timed out")?.map_err(|_|"initialize failed")?};
                let first=tokio::select! {biased;_=cancel.cancelled()=>{let _=service.cancel().await;return Err("cancelled");},r=tokio::time::timeout_at(deadline,service.peer().list_all_tools())=>r};
                let tools=match first {Ok(Ok(tools))=>tools,_=>{let _=service.cancel().await;return Err("tools/list failed");}};
                slot.ready.store(Arc::new(Some(Ready {generation,peer:service.peer().clone(),tools})));slot.state.store(1,Ordering::Release);
                notice(&output,&cancel,Level::Info,format!("mcp {}: ready",slot.name)).await;
                let peer=service.peer().clone();let stop_service=service.cancellation_token();
                let mut completion=Box::pin(service.waiting());let mut finished=false;
                loop {
                    tokio::select! {biased;_=cancel.cancelled()=>break,_=&mut completion=>{finished=true;break;},_=changed.notified()=> {
                        tokio::select!{biased;_=cancel.cancelled()=>break,_=&mut completion=>{finished=true;break;},_=tokio::time::sleep(Duration::from_secs(1))=>{}}
                        while changed.notified().now_or_never().is_some() {}
                        let listed=tokio::select!{biased;_=cancel.cancelled()=>break,_=&mut completion=>{finished=true;break;},r=tokio::time::timeout(Duration::from_secs(10),peer.list_all_tools())=>r};
                        if let Ok(Ok(tools))=listed {slot.ready.store(Arc::new(Some(Ready {generation,peer:peer.clone(),tools})));}
                    }}
                }
                slot.ready.store(Arc::new(None));slot.state.store(2,Ordering::Release);
                stop_service.cancel();if !finished {let _=completion.await;}Err("server exited")
            };
            let outcome:Result<(),&str>=run.await;
            slot.ready.store(Arc::new(None));finish_child(rx,group).await;let _=pipes.cancel_and_join().await;outcome
        }.await;
        slot.ready.store(Arc::new(None));
        if cancel.is_cancelled() {
            return;
        }
        slot.state.store(2, Ordering::Release);
        let excerpt = {
            let ring = stderr.lock().unwrap();
            let start = ring.len().saturating_sub(2048);
            redactor.scrub(&String::from_utf8_lossy(&ring[start..]))
        };
        let reason = result.err().unwrap_or("server exited");
        notice(
            &output,
            &cancel,
            Level::Warn,
            format!("mcp {}: failed: {reason} {excerpt}", slot.name),
        )
        .await;
        if attempt == 4 {
            slot.state.store(3, Ordering::Release);
            notice(
                &output,
                &cancel,
                Level::Error,
                format!("mcp {}: dead", slot.name),
            )
            .await;
            return;
        }
        tokio::select! {biased;_=cancel.cancelled()=>return,_=tokio::time::sleep(Duration::from_secs((1u64<<attempt).min(30)))=>{}}
    }
}
use futures_util::FutureExt;
