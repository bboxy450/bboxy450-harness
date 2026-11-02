// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Projection boundary for the official CLI's own agent loop and external tools.
use super::Internal;
use crate::{
    protocol::*,
    provider::{
        ProviderError,
        claude_code::{self as wire, *},
    },
    redact::Set,
};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use tokio::sync::mpsc;

const INIT_PREFIX: &str = "Claude Code init: ";
#[derive(serde::Serialize, serde::Deserialize)]
struct Init {
    profile: String,
    session_id: String,
    model: String,
    version: String,
}
fn stored_id(log: &[Envelope], profile: &str) -> Option<String> {
    log.iter().rev().find_map(|e| match &e.event {
        Event::ItemCompleted {
            body: Item::Notice { text, .. },
            status: ItemStatus::Completed,
            ..
        } => {
            let init: Init = serde_json::from_str(text.strip_prefix(INIT_PREFIX)?).ok()?;
            (init.profile == profile && valid_id(&init.session_id)).then_some(init.session_id)
        }
        _ => None,
    })
}
fn valid_id(id: &str) -> bool {
    id.len() == 36
        && id.bytes().enumerate().all(|(i, b)| {
            if [8, 13, 18, 23].contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        })
}
struct Projection {
    turn: TurnId,
    generation: u64,
    output: mpsc::Sender<Internal>,
    redactor: Set,
    calls: BTreeMap<String, (ItemId, Item)>,
    completed_calls: BTreeSet<String>,
}
impl Projection {
    async fn emit(&self, event: Event) {
        let _ = self
            .output
            .send(Internal::Event {
                generation: self.generation,
                event,
                cause: None,
            })
            .await;
    }
    fn clean(&self, value: &mut Value) {
        match value {
            Value::String(s) => *s = scrub(&self.redactor, s),
            Value::Array(a) => a.iter_mut().for_each(|v| self.clean(v)),
            Value::Object(o) => {
                for (key, value) in o.iter_mut() {
                    if [
                        "access_token",
                        "refresh_token",
                        "authorization",
                        "api_key",
                        "password",
                        "claude_code_oauth_token",
                    ]
                    .contains(&key.to_ascii_lowercase().as_str())
                    {
                        *value = Value::String("<redacted>".into());
                    } else {
                        self.clean(value);
                    }
                }
            }
            _ => {}
        }
    }
    async fn started(&self, body: &Item) -> ItemId {
        let item = ItemId::new();
        self.emit(Event::ItemStarted {
            turn: Some(self.turn),
            item,
            kind: body.kind(),
            parent: None,
            head: body.head(),
        })
        .await;
        item
    }
    async fn item(&self, body: Item) {
        let item = self.started(&body).await;
        self.emit(Event::ItemCompleted {
            item,
            status: ItemStatus::Completed,
            body,
        })
        .await;
    }
    async fn notice(&self, level: Level, text: impl Into<String>) {
        self.item(Item::Notice {
            level,
            text: scrub(&self.redactor, &text.into()),
        })
        .await;
    }
    async fn blocks(&mut self, event: &Value) -> Result<(), ProviderError> {
        let blocks = event["message"]["content"]
            .as_array()
            .ok_or_else(|| malformed("Claude Code message content is not an array"))?;
        for block in blocks {
            match (event["type"].as_str(), block["type"].as_str()) {
                (Some("assistant"), Some("text")) => {
                    let text = block["text"]
                        .as_str()
                        .ok_or_else(|| malformed("Claude Code text is missing"))?;
                    self.item(Item::AgentMessage {
                        markdown: scrub(&self.redactor, text),
                    })
                    .await;
                }
                (Some("assistant"), Some("thinking")) => {
                    let text = block["thinking"]
                        .as_str()
                        .ok_or_else(|| malformed("Claude Code thinking is missing"))?;
                    self.item(Item::Reasoning {
                        summary: scrub(&self.redactor, text),
                        raw: None,
                        provider_opaque: None,
                    })
                    .await;
                }
                (Some("assistant"), Some("tool_use")) => {
                    let id = block["id"]
                        .as_str()
                        .filter(|s| !s.is_empty())
                        .ok_or_else(|| malformed("Claude Code tool id is missing"))?;
                    let name = block["name"]
                        .as_str()
                        .ok_or_else(|| malformed("Claude Code tool name is missing"))?;
                    if self.calls.contains_key(id) || self.completed_calls.contains(id) {
                        self.notice(Level::Warn, "duplicate Claude Code tool call retired")
                            .await;
                        continue;
                    }
                    if self.completed_calls.len() >= 4096 {
                        return Err(malformed("too many Claude Code tool calls in a turn"));
                    }
                    if self.calls.len() >= 64 {
                        return Err(malformed("too many pending Claude Code tools"));
                    }
                    if !block["input"].is_object() {
                        return Err(malformed("Claude Code tool input is not an object"));
                    }
                    let mut input = block["input"].clone();
                    self.clean(&mut input);
                    let body = Item::ToolCall {
                        call_id: format!("{}:{id}", self.turn),
                        name: format!("claude-code:{}", scrub(&self.redactor, name)),
                        tool_kind: ToolKind::Other,
                        input_raw: input.to_string(),
                        output: vec![],
                        locations: vec![],
                        spill: None,
                    };
                    let item = self.started(&body).await;
                    self.calls.insert(id.into(), (item, body));
                }
                (Some("user"), Some("tool_result")) => {
                    let id = block["tool_use_id"]
                        .as_str()
                        .ok_or_else(|| malformed("Claude Code tool result id is missing"))?;
                    if let Some((item, mut body)) = self.calls.remove(id) {
                        let mut content = block["content"].clone();
                        self.clean(&mut content);
                        let text = content
                            .as_str()
                            .map(str::to_owned)
                            .unwrap_or_else(|| content.to_string());
                        let error = block["is_error"].as_bool().unwrap_or(false);
                        if let Item::ToolCall { output, .. } = &mut body {
                            output.push(if error {
                                OutputBlock::Error { text }
                            } else {
                                OutputBlock::Text { text }
                            });
                        }
                        self.emit(Event::ItemCompleted {
                            item,
                            status: if error {
                                ItemStatus::Failed {
                                    reason: "Claude Code tool failed".into(),
                                }
                            } else {
                                ItemStatus::Completed
                            },
                            body,
                        })
                        .await;
                        self.completed_calls.insert(id.into());
                    } else {
                        self.notice(
                            Level::Warn,
                            "unmatched or duplicate Claude Code tool result retired",
                        )
                        .await;
                    }
                }
                _ => {
                    self.notice(Level::Warn, "unknown Claude Code content block")
                        .await
                }
            }
        }
        Ok(())
    }
    async fn finish(mut self, reason: StopReason, usage: Option<Usage>) {
        let cancelled = matches!(reason, StopReason::Cancelled);
        for (_, (item, mut body)) in std::mem::take(&mut self.calls) {
            if let Item::ToolCall { output, .. } = &mut body {
                output.push(OutputBlock::Error {
                    text: if cancelled {
                        "Claude Code tool cancelled"
                    } else {
                        "Claude Code ended before this tool returned"
                    }
                    .into(),
                });
            }
            self.emit(Event::ItemCompleted {
                item,
                status: if cancelled {
                    ItemStatus::Cancelled
                } else {
                    ItemStatus::Failed {
                        reason: "external tool interrupted".into(),
                    }
                },
                body,
            })
            .await;
        }
        if let Some(usage) = &usage {
            self.item(Item::Usage {
                usage: usage.clone(),
            })
            .await;
        }
        if let StopReason::Error { class, message } = &reason {
            self.notice(Level::Error, format!("{class:?}: {message}"))
                .await;
        }
        self.emit(Event::TurnEnded {
            turn: self.turn,
            reason,
            usage,
        })
        .await;
    }
}

use super::task_group::TaskGroup;
use crate::process::OwnedChild;
use std::{
    collections::VecDeque,
    io,
    os::unix::process::ExitStatusExt,
    path::PathBuf,
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    process::{ChildStdin, ChildStdout, Command},
    sync::{oneshot, watch},
};
use tokio_util::sync::CancellationToken;
const STARTUP: Duration = Duration::from_secs(10);
const STALL: Duration = Duration::from_secs(600);

struct Request {
    turn: TurnId,
    generation: u64,
    blocks: Vec<InputBlock>,
    cancel: CancellationToken,
    done: oneshot::Sender<()>,
}
#[derive(Clone)]
pub(super) struct Handle {
    requests: mpsc::Sender<Request>,
    stop: CancellationToken,
    closed: watch::Receiver<bool>,
    output: mpsc::Sender<Internal>,
}
impl Handle {
    pub(super) fn start(
        options: Options,
        profile: String,
        cwd: PathBuf,
        log: &[Envelope],
        redactor: Set,
        output: mpsc::Sender<Internal>,
        group: &mut TaskGroup,
    ) -> Self {
        let (requests, input) = mpsc::channel(1);
        let stop = group.cancel.child_token();
        let (closed_tx, closed) = watch::channel(false);
        let mut driver = Driver {
            options,
            profile: profile.clone(),
            cwd,
            resume: stored_id(log, &profile),
            redactor,
            output: output.clone(),
            process: None,
            stop: stop.clone(),
        };
        group.spawn(async move {
            driver.run(input).await;
            let _ = closed_tx.send(true);
        });
        Self {
            requests,
            stop,
            closed,
            output,
        }
    }
    pub(super) async fn turn(
        &self,
        turn: TurnId,
        generation: u64,
        blocks: Vec<InputBlock>,
        cancel: CancellationToken,
    ) {
        let (done, wait) = oneshot::channel();
        if self
            .requests
            .send(Request {
                turn,
                generation,
                blocks,
                cancel: cancel.clone(),
                done,
            })
            .await
            .is_err()
        {
            // Close may retire the idle worker before a newly scheduled turn
            // task reaches it. That accepted turn still needs one Cancelled end.
            if self.stop.is_cancelled() || cancel.is_cancelled() {
                let _ = self
                    .output
                    .send(Internal::Event {
                        generation,
                        event: Event::TurnEnded {
                            turn,
                            reason: StopReason::Cancelled,
                            usage: None,
                        },
                        cause: None,
                    })
                    .await;
                return;
            }
            panic!("Claude Code session worker stopped");
        }
        wait.await.expect("Claude Code session worker failed");
    }
    pub(super) async fn close(&mut self) {
        self.stop.cancel();
        let _ = self.closed.wait_for(|closed| *closed).await;
    }
}
struct Driver {
    options: Options,
    profile: String,
    cwd: PathBuf,
    resume: Option<String>,
    redactor: Set,
    output: mpsc::Sender<Internal>,
    process: Option<Process>,
    stop: CancellationToken,
}
impl Driver {
    async fn run(&mut self, mut input: mpsc::Receiver<Request>) {
        loop {
            // Keep draining while idle. Duplicate/late events cannot wait in a
            // pipe until a newer turn is admitted. No frontend owns these pipes.
            let request = if let Some(process) = &mut self.process {
                tokio::select! {
                    biased;
                    _ = self.stop.cancelled() => break,
                    request = input.recv() => request,
                    event = process.next() => {
                        let text = match event {
                            Ok(Some(_)) => "Claude Code event outside a turn retired".to_owned(),
                            _ => {
                                let process = self.process.take().unwrap();
                                process.finish(false, &self.redactor).await.error.message
                            }
                        };
                        let _ = self.output.send(Internal::ServiceNotice { level: Level::Warn, text }).await;
                        tokio::task::yield_now().await;
                        continue;
                    }
                }
            } else {
                tokio::select! { biased; _ = self.stop.cancelled() => break, request = input.recv() => request }
            };
            let Some(request) = request else {
                break;
            };
            self.turn(request).await;
        }
        if let Some(process) = self.process.take() {
            process.finish(true, &self.redactor).await;
        }
    }
    async fn turn(&mut self, request: Request) {
        let mut view = Projection {
            turn: request.turn,
            generation: request.generation,
            output: self.output.clone(),
            redactor: self.redactor.clone(),
            calls: BTreeMap::new(),
            completed_calls: BTreeSet::new(),
        };
        view.item(Item::UserMessage {
            blocks: request.blocks.clone(),
        })
        .await;
        let mut usage = None;
        let result = self.drive(&mut view, &request, &mut usage).await;
        let reason = if request.cancel.is_cancelled() || self.stop.is_cancelled() {
            if let Some(process) = self.process.take() {
                process.finish(true, &self.redactor).await;
            }
            StopReason::Cancelled
        } else {
            match result {
                Ok(()) => StopReason::EndTurn,
                Err(mut error) => {
                    if let Some(process) = self.process.take() {
                        let exit = process.finish(false, &self.redactor).await;
                        if matches!(error.class, ErrorClass::MidStream) {
                            error = exit.error;
                        }
                    }
                    StopReason::Error {
                        class: error.class,
                        message: error.message,
                    }
                }
            }
        };
        view.finish(reason, usage).await;
        let _ = request.done.send(());
    }
    async fn drive(
        &mut self,
        view: &mut Projection,
        request: &Request,
        usage: &mut Option<Usage>,
    ) -> Result<(), ProviderError> {
        let content = request
            .blocks
            .iter()
            .map(|block| match block {
                InputBlock::Text { text } => Ok(json!({"type":"text", "text":text})),
                InputBlock::Mention { path } => {
                    Ok(json!({"type":"text", "text":format!("@{}", path.display())}))
                }
                _ => Err(ProviderError::new(
                    ErrorClass::BadRequest,
                    "Claude Code sessions currently accept text and file mentions",
                )),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut input = serde_json::to_vec(
            &json!({"type":"user", "message":{"role":"user", "content":content}}),
        )
        .map_err(|_| malformed("Claude Code input serialization failed"))?;
        input.push(b'\n');
        if input.len() > MAX_LINE {
            return Err(ProviderError::new(
                ErrorClass::BadRequest,
                "Claude Code input exceeds 1 MiB",
            ));
        }
        if self.process.is_none() {
            check_version(&self.cwd, &self.redactor, &request.cancel, &self.stop).await?;
            if request.cancel.is_cancelled() || self.stop.is_cancelled() {
                return Ok(());
            }
            self.process = Some(
                Process::spawn(&self.options.argv(self.resume.as_deref()), &self.cwd)
                    .map_err(spawn_error)?,
            );
        }
        let process = self.process.as_mut().unwrap();
        // Bounded writes are cancellable even if the CLI stops reading stdin.
        tokio::select! { biased;
            _=request.cancel.cancelled()=>return Ok(()), _=self.stop.cancelled()=>return Ok(()),
            r=tokio::time::timeout(STARTUP,process.stdin.write_all(&input))=>r.map_err(|_| ProviderError::new(ErrorClass::MidStream,"Claude Code stdin timed out"))?.map_err(|_| ProviderError::new(ErrorClass::MidStream,"Claude Code stdin closed"))?,
        }
        loop {
            let process = self.process.as_mut().unwrap();
            if !process.initialized && tokio::time::Instant::now() >= process.init_deadline {
                return Err(ProviderError::new(
                    ErrorClass::Auth,
                    format!("{AUTH_FIX}; no initialization within 10 s"),
                ));
            }
            let timeout = if process.initialized {
                STALL
            } else {
                process
                    .init_deadline
                    .saturating_duration_since(tokio::time::Instant::now())
            };
            let event = tokio::select! { biased;
                _=request.cancel.cancelled()=>return Ok(()), _=self.stop.cancelled()=>return Ok(()),
                r=tokio::time::timeout(timeout,process.next()) => match r {
                    Ok(r)=>r?,
                    Err(_)=>return Err(ProviderError::new(if process.initialized { ErrorClass::Transient } else { ErrorClass::Auth }, if process.initialized { "Claude Code output stalled".into() } else { format!("{AUTH_FIX}; no initialization within 10 s") })),
                },
            };
            let Some(event) = event else {
                return Err(ProviderError::new(
                    ErrorClass::MidStream,
                    "Claude Code stdout closed",
                ));
            };
            if event["uuid"].as_str().is_some_and(|uuid| uuid.len() > 128) {
                return Err(malformed("Claude Code event UUID exceeds 128 bytes"));
            }
            if !process.admit(&event) {
                view.notice(Level::Warn, "duplicate Claude Code event retired")
                    .await;
                continue;
            }
            if let Some(id) = event["session_id"].as_str()
                && let Some(expected) = &self.resume
                && id != expected
            {
                return Err(malformed("Claude Code session id changed unexpectedly"));
            }
            match event["type"].as_str() {
                Some("system") if event["subtype"] == "init" => {
                    let id = event["session_id"]
                        .as_str()
                        .filter(|id| valid_id(id))
                        .ok_or_else(|| malformed("Claude Code init has no valid session id"))?;
                    let model = event["model"]
                        .as_str()
                        .filter(|m| m.len() <= 256)
                        .ok_or_else(|| malformed("Claude Code init has no model"))?;
                    self.resume = Some(id.into());
                    process.initialized = true;
                    let init = Init {
                        profile: self.profile.clone(),
                        session_id: id.into(),
                        model: scrub(&self.redactor, model),
                        version: VERSION.into(),
                    };
                    view.notice(
                        Level::Info,
                        format!("{INIT_PREFIX}{}", serde_json::to_string(&init).unwrap()),
                    )
                    .await;
                }
                Some("assistant" | "user") => {
                    if auth_error(event["error"].as_str().unwrap_or("")) {
                        return Err(ProviderError::new(ErrorClass::Auth, AUTH_FIX));
                    }
                    view.blocks(&event).await?;
                }
                Some("result") => {
                    *usage = Some(wire::usage(&event));
                    let error = event["is_error"]
                        .as_bool()
                        .ok_or_else(|| malformed("Claude Code result lacks is_error"))?;
                    let subtype = event["subtype"]
                        .as_str()
                        .ok_or_else(|| malformed("Claude Code result lacks subtype"))?;
                    if error {
                        if auth_error(&event["errors"].to_string())
                            || auth_error(event["result"].as_str().unwrap_or(""))
                        {
                            return Err(ProviderError::new(ErrorClass::Auth, AUTH_FIX));
                        }
                        // Fixed diagnostic fields only: never reflect raw result bodies.
                        let subtype = scrub(
                            &self.redactor,
                            &subtype.chars().take(80).collect::<String>(),
                        );
                        return Err(ProviderError::new(
                            ErrorClass::BadRequest,
                            format!("Claude Code result: {subtype}"),
                        ));
                    }
                    if subtype != "success" {
                        view.notice(Level::Warn, "unknown Claude Code result subtype")
                            .await;
                    }
                    return Ok(());
                }
                _ => {
                    let kind = event["type"]
                        .as_str()
                        .unwrap_or("missing type")
                        .chars()
                        .take(80)
                        .collect::<String>();
                    view.notice(Level::Warn, format!("unknown Claude Code event: {kind}"))
                        .await;
                }
            }
            tokio::task::yield_now().await;
        }
    }
}
fn spawn_error(error: io::Error) -> ProviderError {
    if error.kind() == io::ErrorKind::NotFound {
        ProviderError::new(
            ErrorClass::Auth,
            format!(
                "claude executable missing; install Claude Code {VERSION}, then run claude /login or claude setup-token"
            ),
        )
    } else {
        ProviderError::new(
            ErrorClass::Config,
            "could not start Claude Code in the session workspace",
        )
    }
}
struct Process {
    child: OwnedChild,
    stdin: ChildStdin,
    stdout: BufReader<ChildStdout>,
    lines: JsonLines,
    stderr: tokio::task::JoinHandle<()>,
    tail: Arc<Mutex<VecDeque<u8>>>,
    seen: BTreeSet<String>,
    order: VecDeque<String>,
    initialized: bool,
    init_deadline: tokio::time::Instant,
}
impl Process {
    fn spawn(args: &[String], cwd: &std::path::Path) -> io::Result<Self> {
        bbt_rt::assert_not_ui_thread();
        // Oracle ruling 01M6ESRQ3W4WEEEZ3F0SWHEZ2R: the published command
        // runner hardcodes null stdin. Reuse the harness's registered owner on
        // bbt_rt rather than forking the toolkit or adding a completion proxy.
        let mut command = Command::new("claude");
        command
            .args(args)
            .current_dir(cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        // No env lookup, override, credential command, or shell interpolation.
        let mut child = OwnedChild::spawn(command)?;
        let stdin = child.stdin().unwrap();
        let stdout = BufReader::new(child.stdout().unwrap());
        let mut pipe = child.stderr().unwrap();
        let tail = Arc::new(Mutex::new(VecDeque::new()));
        let bytes = tail.clone();
        let stderr = bbt_rt::handle().spawn(async move {
            let mut buffer = [0; 2048];
            loop {
                match pipe.read(&mut buffer).await {
                    Ok(0) | Err(_) => break,
                    Ok(n) => {
                        let mut bytes = bytes.lock().unwrap();
                        bytes.extend(&buffer[..n]);
                        let excess = bytes.len().saturating_sub(8192);
                        bytes.drain(..excess);
                    }
                }
            }
        });
        Ok(Self {
            child,
            stdin,
            stdout,
            lines: JsonLines::default(),
            stderr,
            tail,
            seen: BTreeSet::new(),
            order: VecDeque::new(),
            initialized: false,
            init_deadline: tokio::time::Instant::now() + STARTUP,
        })
    }
    fn admit(&mut self, event: &Value) -> bool {
        let Some(uuid) = event["uuid"].as_str() else {
            return true;
        };
        if !self.seen.insert(uuid.into()) {
            return false;
        }
        self.order.push_back(uuid.into());
        if self.order.len() > 4096 {
            self.seen.remove(&self.order.pop_front().unwrap());
        }
        true
    }
    async fn next(&mut self) -> Result<Option<Value>, ProviderError> {
        let pgid = self.child.pgid();
        tokio::select! { biased;
            result = self.lines.next(&mut self.stdout) => result,
            _ = crate::process::leader_exit(pgid) => {
                // Keep already-buffered final frames, but a crashed leader's
                // descendant cannot hold stdout open indefinitely.
                tokio::time::timeout(Duration::from_millis(50), self.lines.next(&mut self.stdout)).await.unwrap_or(Ok(None))
            }
        }
    }
    async fn finish(mut self, interrupt: bool, set: &Set) -> Exit {
        if interrupt {
            self.child.send(libc::SIGINT);
            crate::process::terminate(&self.child).await;
        } else if !self.child.exited().unwrap_or(false) {
            let _ = tokio::time::timeout(
                Duration::from_millis(50),
                crate::process::leader_exit(self.child.pgid()),
            )
            .await;
        }
        let status = self.child.wait().await;
        if tokio::time::timeout(Duration::from_millis(250), &mut self.stderr)
            .await
            .is_err()
        {
            self.stderr.abort();
            let _ = (&mut self.stderr).await;
        }
        let tail: Vec<u8> = self.tail.lock().unwrap().iter().copied().collect();
        let text = String::from_utf8_lossy(&tail);
        let success = status.as_ref().is_ok_and(|s| s.success());
        if auth_error(&text) {
            return Exit {
                success,
                error: ProviderError::new(ErrorClass::Auth, AUTH_FIX),
            };
        }
        let status = status
            .map(|s| format!("code {:?}, signal {:?}", s.code(), s.signal()))
            .unwrap_or_else(|_| "wait failed".into());
        Exit {
            success,
            error: ProviderError::new(
                ErrorClass::MidStream,
                format!(
                    "Claude Code exited ({status}); stderr: {}",
                    scrub(set, &text)
                ),
            ),
        }
    }
}
impl Drop for Process {
    fn drop(&mut self) {
        self.stderr.abort();
    }
}
struct Exit {
    success: bool,
    error: ProviderError,
}
async fn check_version(
    cwd: &std::path::Path,
    set: &Set,
    cancel: &CancellationToken,
    stop: &CancellationToken,
) -> Result<(), ProviderError> {
    let mut process = Process::spawn(&["--version".into()], cwd).map_err(spawn_error)?;
    let mut bytes = vec![];
    let mut stdout = (&mut process.stdout).take(4097);
    let read = tokio::select! { biased;
        _=cancel.cancelled()=>Err(ProviderError::new(ErrorClass::Cancelled,"cancelled")),
        _=stop.cancelled()=>Err(ProviderError::new(ErrorClass::Cancelled,"cancelled")),
        result=tokio::time::timeout(STARTUP,stdout.read_to_end(&mut bytes))=>match result {
            Ok(Ok(_))=>Ok(()),
            Ok(Err(_))=>Err(ProviderError::new(ErrorClass::Config,"Claude Code version read failed")),
            Err(_)=>Err(ProviderError::new(ErrorClass::Auth,format!("{AUTH_FIX}; CLI version check timed out"))),
        },
    };
    let exit = process.finish(read.is_err(), set).await;
    read?;
    if !exit.success {
        return Err(exit.error);
    }
    if bytes.len() > 4096 {
        return Err(ProviderError::new(
            ErrorClass::Config,
            "Claude Code version output is invalid",
        ));
    }
    let version = std::str::from_utf8(&bytes)
        .ok()
        .and_then(|s| s.split_whitespace().next());
    if version != Some(VERSION) {
        return Err(ProviderError::new(
            ErrorClass::Config,
            format!("Claude Code version unsupported; expected {VERSION} (check claude --version)"),
        ));
    }
    Ok(())
}
