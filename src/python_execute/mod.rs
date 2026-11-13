// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Session-owned CPython execution. Model tools, callbacks and output have separate IO.
//! Inspired by Cloudflare Code Mode's host-tool RPC and FastMCP CodeMode's
//! discovery/execution split: https://developers.cloudflare.com/agents/tools/codemode/how-it-works/
//! and https://gofastmcp.com/servers/transforms/code-mode. This implementation
//! uses owned full CPython and existing rmcp clients; no upstream code is copied.
mod process;
mod server;
#[cfg(test)]
mod tests;

use crate::{
    engine::task_group::TaskGroup,
    mcp::{Capture, Manager},
    redact::Set,
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap, io, os::unix::fs::PermissionsExt, path::PathBuf, sync::Arc,
    time::Duration,
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::UnixListener,
    sync::{Mutex, Notify, Semaphore, mpsc},
};
use tokio_util::sync::CancellationToken;
use ulid::Ulid;

pub(crate) const FRAME: usize = 8 * 1024 * 1024;
const OUTPUT: usize = 1024 * 1024;
const CALLS: usize = 4096;
const SOURCE: usize = 1024 * 1024;
/// A run may take a day. Its default is an hour: long enough for a build or a
/// sweep, and a runaway script still ends.
const TIMEOUT: u64 = 24 * 60 * 60 * 1000;
const DEFAULT_TIMEOUT: u64 = 60 * 60 * 1000;
const CALLBACKS_PER_RUN: usize = 8;
/// MCP requests wait at most 25 seconds, below the usual 30-second client deadline.
/// The execution itself keeps its independent durable wall deadline.
const MAX_WAIT_MS: u64 = 25_000;
/// Wait as long as the request may: a run that finishes in time answers in the
/// same call, without a handle and a poll.
const DEFAULT_WAIT_MS: u64 = MAX_WAIT_MS;
type Progress = Option<(Value, mpsc::Sender<Value>)>;

pub(crate) struct Endpoint {
    dir: PathBuf,
    listener: Option<UnixListener>,
}
impl Endpoint {
    pub(crate) async fn new() -> io::Result<Self> {
        // Keep Darwin socket paths short even under a long fixture TMPDIR.
        let dir = PathBuf::from("/tmp").join(format!("bpy-{}", Ulid::new()));
        tokio::fs::create_dir(&dir).await?;
        tokio::fs::set_permissions(&dir, std::fs::Permissions::from_mode(0o700)).await?;
        let socket = dir.join("m");
        let listener = UnixListener::bind(&socket)?;
        tokio::fs::set_permissions(&socket, std::fs::Permissions::from_mode(0o600)).await?;
        Ok(Self {
            dir,
            listener: Some(listener),
        })
    }
    pub(crate) fn socket(&self) -> PathBuf {
        self.dir.join("m")
    }
}
impl Drop for Endpoint {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Chunk {
    stream: String,
    text: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    base64: Option<String>,
}
#[derive(Serialize, Deserialize, Clone)]
struct Child {
    id: Ulid,
    sequence: usize,
    kind: String,
    server: Option<String>,
    tool: Option<String>,
    arguments_digest: String,
    #[serde(default)]
    server_generation: Option<u64>,
    #[serde(default)]
    schema_digest: Option<String>,
    status: String,
    #[serde(default)]
    result: Option<Value>,
    #[serde(default)]
    result_resource: Option<PathBuf>,
}
#[derive(Serialize, Deserialize, Clone)]
struct Record {
    execution_id: Ulid,
    request_digest: String,
    source_digest: String,
    status: String,
    cwd: PathBuf,
    context: Value,
    #[serde(default)]
    pid: Option<u32>,
    #[serde(default)]
    exit_code: Option<i32>,
    #[serde(default)]
    signal: Option<i32>,
    #[serde(default)]
    error: Option<Value>,
    #[serde(default)]
    output: Vec<Chunk>,
    #[serde(default)]
    output_bytes: usize,
    #[serde(default)]
    omitted_bytes: u64,
    #[serde(default)]
    emitted: Vec<Value>,
    #[serde(default)]
    children: Vec<Child>,
}
struct Run {
    record: Mutex<Record>,
    cancel: CancellationToken,
    capture: std::sync::Mutex<Capture>,
    token: String,
    authority: Mutex<Option<Value>>,
    callbacks: Arc<Semaphore>,
    finished: Notify,
}
struct Inner {
    endpoint: Endpoint,
    root: PathBuf,
    cwd: PathBuf,
    mcp: Manager,
    redactor: Set,
    cancel: CancellationToken,
    runs: Mutex<BTreeMap<Ulid, Arc<Run>>>,
    slots: Arc<Semaphore>,
    callbacks: Arc<Semaphore>,
    connections: Arc<Semaphore>,
    executions: Mutex<Vec<tokio::task::JoinHandle<()>>>,
}
impl Run {
    fn capture(&self) -> Capture {
        self.capture
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .clone()
    }
    fn pin_starting_server(&self, server: &str, current: Capture) {
        let mut captured = self.capture.lock().unwrap_or_else(|e| e.into_inner());
        if let (Some(old), Some(ready)) = (captured.get(server), current.get(server)) {
            // Pin a server's first successful initialization, never refresh an
            // established connection/schema or permit a failed-generation retry.
            if old.1.is_empty() && !ready.1.is_empty() && (old.0 == 0 || old.0 == ready.0) {
                captured.insert(server.to_owned(), ready.clone());
            }
        }
    }
}
#[derive(Clone)]
pub(crate) struct Service(Arc<Inner>);

fn digest(value: &impl Serialize) -> String {
    let mut v = serde_json::to_value(value).unwrap_or(Value::Null);
    v.sort_all_objects();
    format!(
        "{:x}",
        Sha256::digest(serde_json::to_vec(&v).unwrap_or_default())
    )
}
pub(crate) fn fault(code: &str, message: &str) -> Value {
    json!({"error":{"code":code,"message":message},"hints":[{"reason":"Read execution state and nested receipts before retrying any effect.","method":"tools/call","params":{"name":"python_execute_orient","arguments":{}}}]})
}
fn envelope(value: Value) -> Value {
    let failed = value.get("error").is_some_and(|v| !v.is_null());
    json!({"content":[{"type":"text","text":value.to_string()}],"structuredContent":value,"isError":failed})
}
fn id(value: &Value, key: &str) -> Result<Ulid, Value> {
    value[key]
        .as_str()
        .and_then(|s| s.parse().ok())
        .filter(|id| *id != Ulid::nil())
        .ok_or_else(|| fault("invalid_arguments", &format!("{key} must be a nonnil ULID")))
}
impl Service {
    fn normalize_context(&self, mut context: Value) -> Result<Value, Value> {
        if !context.is_object() {
            return Err(fault(
                "invalid_context",
                "current execution context is not an object",
            ));
        }
        if context["cwd"].is_null() {
            context["cwd"] = json!(self.0.cwd);
        }
        if context["target"].is_null() {
            context["target"] = json!({"kind":"local"});
        }
        Ok(context)
    }
    pub(crate) async fn start(
        mut endpoint: Endpoint,
        root: PathBuf,
        cwd: PathBuf,
        mcp: Manager,
        redactor: Set,
        services: &mut TaskGroup,
    ) -> io::Result<Self> {
        tokio::fs::create_dir_all(&root).await?;
        tokio::fs::set_permissions(&root, std::fs::Permissions::from_mode(0o700)).await?;
        let listener = endpoint
            .listener
            .take()
            .ok_or_else(|| io::Error::other("runtime listener already consumed"))?;
        let service = Self(Arc::new(Inner {
            endpoint,
            root,
            cwd,
            mcp,
            redactor,
            cancel: services.cancel.child_token(),
            runs: Mutex::new(BTreeMap::new()),
            slots: Arc::new(Semaphore::new(4)),
            callbacks: Arc::new(Semaphore::new(32)),
            connections: Arc::new(Semaphore::new(256)),
            executions: Mutex::new(vec![]),
        }));
        service.recover().await?;
        let s = service.clone();
        services.spawn(async move {
            let mut tasks = tokio::task::JoinSet::new();
            loop {
                tokio::select! { biased;
                    _ = s.0.cancel.cancelled() => break,
                    Some(_) = tasks.join_next(), if !tasks.is_empty() => {},
                    connection = listener.accept() => {
                        let Ok((stream, _)) = connection else { break };
                        if !stream.peer_cred().is_ok_and(|p|p.uid() == unsafe {libc::geteuid()}) { continue; }
                        let Ok(permit) = s.0.connections.clone().try_acquire_owned() else {
                            let mut stream=stream;
                            let mut bytes=serde_json::to_vec(&json!({"jsonrpc":"2.0","id":null,"error":{"code":-32000,"message":"Python runtime connection capacity busy","data":fault("runtime_busy","Connection was not admitted; inspect existing state before retrying effects")}})).unwrap_or_default();bytes.push(b'\n');
                            let _=tokio::time::timeout(Duration::from_millis(20),stream.write_all(&bytes)).await;
                            continue;
                        };
                        let current = s.clone();
                        tasks.spawn(async move { let _permit=permit; current.serve(stream).await; });
                    }
                }
            }
            // Every executor/callback observes this cancellation before any new effect.
            s.0.cancel.cancel();
            let runs:Vec<_> = s.0.runs.lock().await.values().cloned().collect();
            for run in runs {run.cancel.cancel();}
            while tasks.join_next().await.is_some() {}
            let executions=std::mem::take(&mut *s.0.executions.lock().await);
            for execution in executions {let _=execution.await;}
        });
        Ok(service)
    }
    async fn recover(&self) -> io::Result<()> {
        let mut dir = tokio::fs::read_dir(&self.0.root).await?;
        while let Some(entry) = dir.next_entry().await? {
            let path = entry.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            let Some(name) = path.file_stem().and_then(|s| s.to_str()) else {
                continue;
            };
            if name.parse::<Ulid>().is_err() {
                continue;
            }
            let bytes = tokio::fs::read(&path).await?;
            let mut record: Record = match serde_json::from_slice(&bytes) {
                Ok(record) => record,
                Err(_) => {
                    // Keep the original bytes as evidence and reserve the old
                    // identity as unknown. One torn/corrupt record cannot stop
                    // unrelated sessions or authorize replay of its source.
                    let backup = self.0.root.join(format!(".{name}.corrupt-{}", Ulid::new()));
                    tokio::fs::rename(&path, &backup).await?;
                    Record {
                        execution_id: name.parse().map_err(io::Error::other)?,
                        request_digest: "unrecoverable".into(),
                        source_digest: String::new(),
                        status: "unknown".into(),
                        cwd: self.0.cwd.clone(),
                        context: Value::Null,
                        pid: None,
                        exit_code: None,
                        signal: None,
                        error: Some(fault(
                            "execution_record_corrupt",
                            &format!(
                                "Original record preserved at {}; no execution was replayed",
                                backup.display()
                            ),
                        )),
                        output: vec![],
                        output_bytes: 0,
                        omitted_bytes: 0,
                        emitted: vec![],
                        children: vec![],
                    }
                }
            };
            if matches!(
                record.status.as_str(),
                "starting" | "running" | "cancelling"
            ) {
                record.status = "unknown".into();
                for child in &mut record.children {
                    if child.status == "dispatching" {
                        child.status = "unknown".into();
                    }
                }
                record.error = Some(fault(
                    "execution_interrupted",
                    "Prior process custody was lost; no script or nested effect was replayed",
                ));
                self.save_record(&record).await?;
            }
            if record.request_digest == "unrecoverable" {
                self.save_record(&record).await?;
            }
        }
        Ok(())
    }
    async fn save_record(&self, record: &Record) -> io::Result<()> {
        let path = self.0.root.join(format!("{}.json", record.execution_id));
        let temporary = self.0.root.join(format!(".{}.tmp", Ulid::new()));
        let bytes = serde_json::to_vec(record).map_err(io::Error::other)?;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .await?;
        file.write_all(&bytes).await?;
        file.sync_all().await?;
        tokio::fs::rename(temporary, path).await?;
        let root = self.0.root.clone();
        crate::runtime::blocking(move || std::fs::File::open(root)?.sync_all()).await
    }
    async fn persist(&self, run: &Run) -> io::Result<()> {
        self.save_record(&*run.record.lock().await).await
    }
    async fn get_record(&self, execution: Ulid) -> Result<Record, Value> {
        if let Some(run) = self.0.runs.lock().await.get(&execution).cloned() {
            return Ok(run.record.lock().await.clone());
        }
        let bytes = tokio::fs::read(self.0.root.join(format!("{execution}.json")))
            .await
            .map_err(|_| {
                fault(
                    "execution_not_found",
                    "execution is not in this harness session",
                )
            })?;
        serde_json::from_slice(&bytes)
            .map_err(|_| fault("execution_corrupt", "execution record could not be read"))
    }
    async fn check_context(&self, run: &Run) -> Result<Value, Value> {
        if run.cancel.is_cancelled() || self.0.cancel.is_cancelled() {
            return Err(fault("cancelled", "execution or harness session is closed"));
        }
        let current = self
            .0
            .mcp
            .execution_context(&run.capture(), &run.cancel)
            .await
            .map_err(|cause| {
                let mut error=fault("context_unavailable","Current native admission is unavailable; inspect the preserved cause and recovery hints");
                error["cause"]=cause;error
            })?;
        if let Some(context) = &current
            && context["target"]["kind"]
                .as_str()
                .is_some_and(|kind| kind != "local")
        {
            return Err(fault(
                "unsupported_target",
                "this CPython runtime requires the authenticated local target",
            ));
        }
        let stable = self.normalize_context(
            current.unwrap_or_else(|| json!({"managed":false,"cwd":self.0.cwd})),
        )?;
        let mut expected = run.authority.lock().await;
        if let Some(old) = expected.as_ref() {
            if old != &stable {
                return Err(fault(
                    "execution_context_changed",
                    "seat/grant/workspace/KB binding changed; no new effect was sent",
                ));
            }
        } else {
            *expected = Some(stable.clone());
        }
        Ok(stable)
    }
    async fn execute(&self, args: Value, context: Value) -> Result<Value, Value> {
        let source = args["source"]
            .as_str()
            .filter(|s| !s.is_empty() && s.len() <= SOURCE)
            .ok_or_else(|| {
                fault(
                    "invalid_arguments",
                    "source must contain 1..1048576 UTF-8 bytes",
                )
            })?
            .to_owned();
        let timeout = args["timeout_ms"].as_u64().unwrap_or(DEFAULT_TIMEOUT);
        if !(10..=TIMEOUT).contains(&timeout) {
            return Err(fault(
                "invalid_arguments",
                "timeout_ms must be 10..86400000 (24 hours)",
            ));
        }
        let execution = if args.get("op_id").is_some() {
            id(&args, "op_id")?
        } else {
            Ulid::new()
        };
        let request_digest = digest(
            &json!({"source":source,"inputs":args.get("inputs").cloned().unwrap_or_else(||json!({})),"timeout_ms":timeout,"context":context}),
        );
        if let Ok(old) = self.get_record(execution).await {
            if old.request_digest != request_digest {
                return Err(fault(
                    "op_id_reused",
                    "operation ID already names different Python source/inputs/context",
                ));
            }
            return Ok(self.view(&old, 0));
        }
        let permit = self.0.slots.clone().try_acquire_owned().map_err(|_| {
            fault(
                "execution_limit",
                "four Python executions are already active in this session",
            )
        })?;
        let run = Arc::new(Run {
            record: Mutex::new(Record {
                execution_id: execution,
                request_digest,
                source_digest: digest(&source),
                status: "starting".into(),
                cwd: self.0.cwd.clone(),
                context: context.clone(),
                pid: None,
                exit_code: None,
                signal: None,
                error: None,
                output: vec![],
                output_bytes: 0,
                omitted_bytes: 0,
                emitted: vec![],
                children: vec![],
            }),
            cancel: self.0.cancel.child_token(),
            capture: std::sync::Mutex::new(self.0.mcp.capture()),
            token: Ulid::new().to_string(),
            authority: Mutex::new(Some(context)),
            callbacks: Arc::new(Semaphore::new(CALLBACKS_PER_RUN)),
            finished: Notify::new(),
        });
        // Publish complete, fsynced bytes with no-replace. A crash cannot leave
        // a partially written final admission, and a competing client cannot
        // replace the winner. Orphan temporary files are never executable.
        let path = self.0.root.join(format!("{execution}.json"));
        let temporary = self
            .0
            .root
            .join(format!(".admit-{execution}-{}.tmp", Ulid::new()));
        let bytes = serde_json::to_vec(&*run.record.lock().await)
            .map_err(|_| fault("execution_storage", "cannot serialize admission"))?;
        let mut file = tokio::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&temporary)
            .await
            .map_err(|_| fault("execution_storage", "cannot stage admission"))?;
        file.write_all(&bytes)
            .await
            .map_err(|_| fault("execution_storage", "cannot stage admission"))?;
        file.sync_all()
            .await
            .map_err(|_| fault("execution_storage", "cannot persist staged admission"))?;
        let publication = tokio::fs::hard_link(&temporary, &path).await;
        let _ = tokio::fs::remove_file(temporary).await;
        match publication {
            Ok(()) => {
                let root = self.0.root.clone();
                crate::runtime::blocking(move || std::fs::File::open(root)?.sync_all())
                    .await
                    .map_err(|_| {
                        fault("execution_storage", "cannot persist admission publication")
                    })?;
            }
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {
                let old = self.get_record(execution).await?;
                if old.request_digest != run.record.lock().await.request_digest {
                    return Err(fault(
                        "op_id_reused",
                        "operation ID already names different source",
                    ));
                }
                return Ok(self.view(&old, 0));
            }
            Err(_) => {
                return Err(fault(
                    "execution_storage",
                    "cannot persist execution before dispatch",
                ));
            }
        }
        let mut executions = self.0.executions.lock().await;
        if self.0.cancel.is_cancelled() {
            {
                let mut record = run.record.lock().await;
                record.status = "cancelled".into();
            }
            self.persist(&run)
                .await
                .map_err(|_| fault("execution_storage", "cannot persist cancelled admission"))?;
            return Ok(self.view(&*run.record.lock().await, 0));
        }
        self.0.runs.lock().await.insert(execution, run.clone());
        let service = self.clone();
        let current = run.clone();
        let task = bbt_rt::handle().spawn(async move {
            let _permit = permit;
            let result = service
                .drive(
                    &current,
                    source,
                    args.get("inputs").cloned().unwrap_or_else(|| json!({})),
                    timeout,
                )
                .await;
            let was_cancelled = current.cancel.is_cancelled();
            // Retire callback admission on every terminal outcome. A delayed
            // socket or a copied token cannot reopen a completed execution.
            current.cancel.cancel();
            if let Err(error) = result {
                let mut record = current.record.lock().await;
                record.status = if was_cancelled { "cancelled" } else { "failed" }.into();
                record.error = Some(error);
            }
            let _ = service.persist(&current).await;
            current.finished.notify_waiters();
        });
        executions.retain(|task| !task.is_finished());
        executions.push(task);
        drop(executions);
        Ok(self.view(&*run.record.lock().await, 0))
    }
    fn view(&self, record: &Record, offset: usize) -> Value {
        let mut end = offset.min(record.output.len());
        let mut bytes = 0;
        while end < record.output.len() && end - offset < 32 {
            let size = record.output[end].text.len()
                + record.output[end].base64.as_ref().map_or(0, String::len);
            if end > offset && bytes + size > 64 * 1024 {
                break;
            }
            bytes += size;
            end += 1;
        }
        json!({"execution_id":record.execution_id,"status":record.status,"cwd":record.cwd,
            "context":record.context,"pid":record.pid,"exit_code":record.exit_code,"signal":record.signal,
            "output":record.output.get(offset..end).unwrap_or_default(),"next_cursor":(end<record.output.len() || matches!(record.status.as_str(),"starting"|"running"|"cancelling")).then(||end.to_string()),
            "omitted_bytes":record.omitted_bytes,"emitted":record.emitted,"children":record.children.iter().map(|c|json!({"id":c.id,"sequence":c.sequence,"kind":c.kind,"server":c.server,"tool":c.tool,"arguments_digest":c.arguments_digest,"server_generation":c.server_generation,"schema_digest":c.schema_digest,"status":c.status,"result_cursor":format!("child:{}:0",c.sequence)})).collect::<Vec<_>>(),
            "error":record.error.as_ref().map(|failure|failure.get("error").unwrap_or(failure)),
            "failure":record.error,"hints":record.error.as_ref().and_then(|failure|failure.get("hints")).cloned().unwrap_or_else(||json!([])),"retry_script":false})
    }
    async fn dispatch(
        &self,
        name: &str,
        args: Value,
        prefetched: Option<Value>,
        expected: Option<Value>,
        progress: Progress,
    ) -> Value {
        let started = tokio::time::Instant::now();
        let allowed: &[&str] = match name {
            "python_execute" => &["source", "inputs", "timeout_ms", "wait_ms", "op_id"],
            "python_execution_get" => &["execution_id", "cursor", "wait_ms"],
            "python_execution_cancel" => &["execution_id", "op_id"],
            "python_execute_orient" => &[],
            _ => return envelope(fault("tool_unavailable", "unknown Python runtime tool")),
        };
        let Some(object) = args.as_object() else {
            return envelope(fault(
                "invalid_arguments",
                "arguments must be a JSON object",
            ));
        };
        if let Some(unknown) = object.keys().find(|key| !allowed.contains(&key.as_str())) {
            return envelope(fault(
                "invalid_arguments",
                &format!(
                    "unknown argument {unknown:?}; {name} takes {}",
                    if allowed.is_empty() {
                        "no arguments".to_owned()
                    } else {
                        allowed.join(", ")
                    }
                ),
            ));
        }
        if object
            .get("timeout_ms")
            .is_some_and(|v| v.as_u64().is_none())
            || object.get("wait_ms").is_some_and(|v| v.as_u64().is_none())
            || object
                .get("cursor")
                .is_some_and(|v| v.as_str().is_none_or(|s| s.len() > 256))
        {
            return envelope(fault(
                "invalid_arguments",
                "timeout_ms and wait_ms must be nonnegative integers; cursor must be a string of at most 256 bytes",
            ));
        }
        if object.contains_key("op_id") && id(&args, "op_id").is_err() {
            return envelope(fault("invalid_arguments", "op_id must be a nonnil ULID"));
        }
        // The native bridge supplies a freshly admitted context on the proxy
        // door. Querying its same synchronous stdio connection while that door
        // waits for this reply would deadlock. Other MCP clients have their own
        // runtime connection and resolve through the ordinary native client.
        let proxy = prefetched.is_some();
        let context = if name == "python_execution_cancel" {
            Ok(Value::Null)
        } else {
            match prefetched {
                Some(context) => self.normalize_context(context),
                None => self.frontend_context().await,
            }
        };
        let context = match context {
            Ok(context) => context,
            Err(error) if name != "python_execution_cancel" => return envelope(error),
            Err(_) => Value::Null,
        };
        if name != "python_execution_cancel" && expected.is_some_and(|expected| expected != context)
        {
            return envelope(fault(
                "execution_context_changed",
                "Nested execution context changed before admission; no new script was started",
            ));
        }
        let wait_ms = args["wait_ms"]
            .as_u64()
            .unwrap_or(DEFAULT_WAIT_MS)
            .min(MAX_WAIT_MS);
        // Reserve the existing bounded publication-context query inside the cap.
        let deadline = (started + Duration::from_millis(MAX_WAIT_MS - 2_000))
            .min(tokio::time::Instant::now() + Duration::from_millis(wait_ms));
        let get_args = args.clone();
        let result = match name {
            "python_execute" => self.execute(args, context.clone()).await,
            "python_execution_get" => self.get(args, context.clone()).await,
            "python_execution_cancel" => match id(&args, "execution_id") {
                Ok(id) => {
                    let run = self.0.runs.lock().await.get(&id).cloned();
                    if let Some(run) = run {
                        let mut r = run.record.lock().await;
                        if matches!(r.status.as_str(), "starting" | "running") {
                            r.status = "cancelling".into();
                        }
                        let persisted = self.save_record(&r).await;
                        run.cancel.cancel();
                        if persisted.is_err() {
                            return envelope(fault(
                                "cancellation_unknown",
                                "Cancellation was requested but its durable receipt is unavailable; inspect execution state",
                            ));
                        }
                        Ok(
                            json!({"execution_id":id,"status":r.status,"cancel_requested":true,"retry_script":false}),
                        )
                    } else {
                        self.get_record(id).await.map(|r|json!({"execution_id":id,"status":r.status,"cancel_requested":false,"retry_script":false}))
                    }
                }
                Err(e) => Err(e),
            },
            "python_execute_orient" => Ok(self.orientation(context.clone()).await),
            _ => Err(fault("tool_unavailable", "unknown Python runtime tool")),
        };
        let mut result = result.unwrap_or_else(|e| e);
        if matches!(name, "python_execute" | "python_execution_get")
            && let Some(execution) = result["execution_id"].as_str().and_then(|s| s.parse().ok())
            && matches!(
                result["status"].as_str(),
                Some("starting" | "running" | "cancelling")
            )
            && !get_args["cursor"]
                .as_str()
                .is_some_and(|s| s.starts_with("child:"))
        {
            self.wait(execution, deadline, progress).await;
            let mut args = if name == "python_execute" {
                json!({"execution_id":execution})
            } else {
                get_args
            };
            args.as_object_mut().unwrap().remove("wait_ms");
            result = self.get(args, context.clone()).await.unwrap_or_else(|e| e);
        }
        // Proxy callers bracket their synchronous bridge request themselves.
        // Direct runtime MCP clients need the same publication fence here.
        if !proxy && name != "python_execution_cancel" {
            match self.frontend_context().await {
                Ok(current) if current == context => {}
                _ => {
                    let mut error = fault(
                        "execution_context_changed",
                        "authority/source/KB changed before publication; reconcile the retained execution handle",
                    );
                    error["execution_id"] = result["execution_id"].clone();
                    return envelope(error);
                }
            }
        }
        envelope(result)
    }
    async fn wait(&self, execution: Ulid, deadline: tokio::time::Instant, progress: Progress) {
        let Some(run) = self.0.runs.lock().await.get(&execution).cloned() else {
            return;
        };
        let mut tick = tokio::time::interval(Duration::from_secs(1));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut elapsed = 0;
        loop {
            // Register before inspecting state so completion cannot be lost.
            let finished = run.finished.notified();
            tokio::pin!(finished);
            finished.as_mut().enable();
            if !matches!(
                run.record.lock().await.status.as_str(),
                "starting" | "running" | "cancelling"
            ) {
                return;
            }
            tokio::select! { biased;
                _ = tokio::time::sleep_until(deadline) => return,
                _ = self.0.cancel.cancelled() => return,
                _ = &mut finished => {},
                _ = tick.tick(), if progress.is_some() => {
                    if let Some((token, sender)) = &progress {
                        let _ = sender.try_send(json!({"jsonrpc":"2.0","method":"notifications/progress","params":{"progressToken":token,"progress":elapsed,"message":format!("Python execution {execution} is running")}}));
                        elapsed += 1;
                    }
                }
            }
        }
    }
    async fn frontend_context(&self) -> Result<Value, Value> {
        let current = tokio::time::timeout(
            Duration::from_secs(2),
            self.0
                .mcp
                .execution_context(&self.0.mcp.capture(), &self.0.cancel),
        )
        .await
        .map_err(|_| {
            fault(
                "context_pending",
                "native context is not ready; retry orientation, not a mutating script",
            )
        })?
        .map_err(|e| json!({"error":e,"hints":fault("context_unavailable","Read the preserved current-admission error before retrying") ["hints"]}))?;
        self.normalize_context(current.unwrap_or_else(|| json!({"managed":false,"cwd":self.0.cwd})))
    }
    async fn orientation(&self, context: Value) -> Value {
        let interpreter = process::resolve("python3").await;
        let version = if let Some(path) = &interpreter {
            crate::process::run(
                crate::process::Request {
                    program: path.to_string_lossy().into(),
                    args: vec!["--version".into()],
                    stdin: None,
                    cwd: self.0.cwd.clone(),
                    timeout: Duration::from_secs(2),
                    spill_dir: self.0.root.clone(),
                    cap: 256,
                    redactor: self.0.redactor.clone(),
                },
                self.0.cancel.child_token(),
            )
            .await
            .ok()
            .filter(|out| out.code == Some(0))
            .map(|out| String::from_utf8_lossy(&out.bytes).trim().to_owned())
        } else {
            None
        };
        let local = context["target"]["kind"] == "local";
        json!({"ready":version.is_some()&&local,"interpreter":interpreter,"version":version,"runtime":"owned CPython process; standard library and top-level await; ordinary host execution authority",
            "cwd":context["cwd"],"context":context,"mcp_servers":self.0.mcp.servers(),"limits":{"source_bytes":SOURCE,"timeout_ms":TIMEOUT,"default_timeout_ms":DEFAULT_TIMEOUT,"wait_ms":MAX_WAIT_MS,"default_wait_ms":DEFAULT_WAIT_MS,"nested_call_timeout_ms":crate::mcp::NESTED_CALL_TIMEOUT_MS,"retained_output_bytes":OUTPUT,"callback_frame_bytes":FRAME,"nested_calls":CALLS,"active_executions":4,"active_callbacks_per_execution":CALLBACKS_PER_RUN,"connections":256},
            "policy":"Original MCP server authorization and credentials apply; parent Codex/Claude hooks are not automatically invoked per nested call. Python and subprocess use normal harness host authority, with durable nested receipts.",
            "bindings":{
                "tools as functions":["bboxy.<tool>(key=value, ...)  # the host's tools; returns the tool's data, raises ToolError","tools.<server>.<tool>(...)  # any MCP server","tools.search('words')  # find a tool: [{server, name, signature, description}]","bboxy.tools() / bboxy.describe('knowledge_write')  # signatures and the exact input schema","_timeout_ms= and _op_id= bound and name one call; every other keyword reaches the tool","await is optional on all of the above"],
                "result":["a final bare expression is the result","emit(value)  # more results; a value too large to return is spilled to a file as {spilled, bytes, head}","print(...) is kept as output"],
                "shell and files":["sh('git status --porcelain').out  # .out .err .code .ok .json(); check=True raises","fs.replace(path, old, new, count=1)  # refuses and says what it found unless old occurs exactly count times","fs.lines(path, start, end)  # numbered lines","commands.run([executable, *args], input=None, json_input=value, cwd=None, check=False)  # exact argv, async"],
                "raw MCP":["await mcp.servers()","await mcp.tools(server, cursor=None)","await mcp.call(server, tool, arguments, op_id=None, timeout_ms=25000)  # the full envelope; .data is what the tool returned; use with asyncio.gather for overlapping calls"],
                "python":["inputs","execution","workspace"]},
            "examples":["open_tasks = bboxy.task_list(status='open')\n[t.title for t in open_tasks['items']]","for seat in bboxy.seat_list()['items']:\n    if seat.record.lifecycle != 'open':\n        continue\n    emit({'seat': seat.record.name, 'tasks': len(seat.record.tasks)})","hits = sh(\"git grep -n 'fn main' -- '*.rs'\").out.splitlines()\nlen(hits)","found = tools.search('close seat')\n[(t.server, t.signature) for t in found]"],
            "workflow":"Discover your native server's current tools. Read knowledge_orient before KB edits; workspace_orient describes provisioning/selecting a source workspace when your current capability permits it. Existing managed writers retain their admitted source cwd and canonical KB. Use knowledge_commit for a substantial checkpoint, sharing or acceptance, rather than each draft edit.",
            "recovery":"Supply a caller-generated ULID op_id before effectful execution when handle recovery matters. Read python_execution_get and original nested operation IDs. Do not replay source with a new ID after lost output or unknown effects.",
            "setup_error":if !local {fault("unsupported_target","This runtime requires the authenticated local execution target")}else if version.is_some(){Value::Null}else{fault("python_unavailable","Configure a working Python 3 in the existing harness PATH; no package installation is performed")}})
    }
    async fn get(&self, args: Value, context: Value) -> Result<Value, Value> {
        use tokio::io::{AsyncReadExt, AsyncSeekExt};
        let execution = id(&args, "execution_id")?;
        let record = self.get_record(execution).await?;
        if !record.context.is_null() && record.context != context {
            let mut error = fault(
                "execution_context_changed",
                "current authority/source/KB no longer matches this execution; results were not republished",
            );
            error["execution_id"] = json!(execution);
            error["child_request_ids"] = json!(
                record
                    .children
                    .iter()
                    .map(|child| child.id)
                    .collect::<Vec<_>>()
            );
            return Err(error);
        }
        let cursor = args["cursor"].as_str().unwrap_or("0");
        if cursor.starts_with("child:") {
            let parts: Vec<_> = cursor.split(':').collect();
            if parts.len() != 3 {
                return Err(fault("invalid_cursor", "invalid child result cursor"));
            }
            let sequence = parts[1]
                .parse::<usize>()
                .map_err(|_| fault("invalid_cursor", "invalid child receipt number"))?;
            let offset = parts[2]
                .parse::<usize>()
                .map_err(|_| fault("invalid_cursor", "invalid child result offset"))?;
            let child = record
                .children
                .get(sequence)
                .ok_or_else(|| fault("invalid_cursor", "child receipt does not exist"))?;
            let (bytes, total) = if let Some(path) = &child.result_resource {
                let mut f = tokio::fs::File::open(path).await.map_err(|_| {
                    fault("execution_storage", "nested result resource unavailable")
                })?;
                let total = f
                    .metadata()
                    .await
                    .map_err(|_| fault("execution_storage", "cannot inspect nested result"))?
                    .len() as usize;
                if offset > total {
                    return Err(fault(
                        "invalid_cursor",
                        "child result offset exceeds stored bytes",
                    ));
                }
                f.seek(std::io::SeekFrom::Start(offset as u64))
                    .await
                    .map_err(|_| fault("invalid_cursor", "cannot seek nested result"))?;
                let mut bytes = vec![0; (total - offset).min(32 * 1024)];
                f.read_exact(&mut bytes)
                    .await
                    .map_err(|_| fault("execution_storage", "cannot read nested result page"))?;
                (bytes, total)
            } else {
                let bytes = serde_json::to_vec(&child.result)
                    .map_err(|_| fault("execution_storage", "cannot encode child result"))?;
                if offset > bytes.len() {
                    return Err(fault(
                        "invalid_cursor",
                        "child result offset exceeds stored bytes",
                    ));
                }
                let total = bytes.len();
                (
                    bytes[offset..(offset + 32 * 1024).min(total)].to_vec(),
                    total,
                )
            };
            let next = offset + bytes.len();
            return Ok(
                json!({"execution_id":execution,"child_request_id":child.id,"status":child.status,"encoding":"base64-json","data_base64":STANDARD.encode(bytes),"total_bytes":total,"next_cursor":(next<total).then(||format!("child:{sequence}:{next}"))}),
            );
        }
        let offset = cursor
            .parse::<usize>()
            .map_err(|_| fault("invalid_cursor", "invalid output cursor"))?;
        if offset > record.output.len() {
            return Err(fault(
                "invalid_cursor",
                "output cursor is outside this execution",
            ));
        }
        Ok(self.view(&record, offset))
    }
    async fn drive(
        &self,
        run: &Arc<Run>,
        source: String,
        inputs: Value,
        timeout: u64,
    ) -> Result<(), Value> {
        // Unrelated slow or invalid MCP servers must not delay local Python.
        // Already-ready generations were pinned at admission; a still-starting
        // server is pinned on first discovery/call without retrying old bindings.
        let context = self.check_context(run).await?;
        let cwd = context["cwd"]
            .as_str()
            .map(PathBuf::from)
            .unwrap_or_else(|| self.0.cwd.clone());
        {
            let mut record = run.record.lock().await;
            record.cwd = cwd.clone();
            record.context = context.clone();
        }
        self.persist(run).await.map_err(|_| {
            fault(
                "execution_storage",
                "cannot persist context before starting Python",
            )
        })?;
        let program = process::resolve("python3").await.ok_or_else(|| {
            fault(
                "python_unavailable",
                "Python 3 is not available in the harness PATH",
            )
        })?;
        let launch = json!({"socket":self.0.endpoint.socket(),"execution_id":run.record.lock().await.execution_id,"callback_token":run.token,"source":source,"inputs":inputs,"context":context});
        let mut command = tokio::process::Command::new(program);
        command.args(["-I", "-u", "-c", include_str!("worker.py")]);
        let output = self
            .capture(
                run,
                command,
                cwd,
                Some(
                    serde_json::to_vec(&launch)
                        .map_err(|_| fault("invalid_arguments", "Python inputs are not JSON"))?,
                ),
                timeout,
                true,
            )
            .await?;
        self.check_context(run).await.or_else(|error| {
            if run.cancel.is_cancelled() {
                Ok(Value::Null)
            } else {
                Err(error)
            }
        })?;
        let mut record = run.record.lock().await;
        for child in &mut record.children {
            if child.status == "dispatching" {
                child.status = "unknown".into();
            }
        }
        record.exit_code = output.code;
        record.signal = output.signal;
        record.status = if output.cancelled {
            "cancelled"
        } else if output.timed_out {
            "timed_out"
        } else if output.code != Some(0) {
            "failed"
        } else if record.children.iter().any(|c| c.status != "succeeded") {
            "completed_with_errors"
        } else {
            "completed"
        }
        .into();
        if output.timed_out {
            record.error = Some(fault(
                "execution_timeout",
                "Python exceeded its wall deadline; earlier effects may have completed",
            ));
        } else if !output.cancelled && output.code != Some(0) && record.error.is_none() {
            let mut error = fault(
                "python_failed",
                "Python exited unsuccessfully; inspect retained stderr and original nested receipts before retrying any effect",
            );
            error["exit_code"] = json!(output.code);
            error["signal"] = json!(output.signal);
            record.error = Some(error);
        }
        Ok(())
    }
    async fn append(&self, run: &Run, stream: &str, bytes: Vec<u8>) {
        if bytes.is_empty() {
            return;
        }
        let mut record = run.record.lock().await;
        let room = OUTPUT.saturating_sub(record.output_bytes);
        let checkpoint = record.output_bytes / (64 * 1024);
        let take = bytes.len().min(room);
        record.omitted_bytes += (bytes.len() - take) as u64;
        if take > 0 {
            record.output.push(Chunk {
                stream: stream.into(),
                text: String::from_utf8_lossy(&bytes[..take]).into(),
                base64: std::str::from_utf8(&bytes[..take])
                    .is_err()
                    .then(|| STANDARD.encode(&bytes[..take])),
            });
            record.output_bytes += take;
        }
        if record.output_bytes / (64 * 1024) != checkpoint {
            let _ = self.save_record(&record).await;
        }
    }
}
