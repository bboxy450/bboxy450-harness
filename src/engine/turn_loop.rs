// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::{
    Internal, history,
    task_group::{TaskGroup, send},
};
use crate::{
    protocol::*,
    provider::{Finish, ModelEvent, ModelRequest, Provider, ToolChoice, WireMessage},
    redact::Set,
    store::Blobs,
    tools::{Registry, ToolOutput},
};
use bbt_yarn::{reactor, reactor::Control, source};
use futures_util::{FutureExt, StreamExt};
use serde_json::{Value, json};
use std::{
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
/// One declaration is consumed by both reactor! and reactor_model!. Tool and
/// provider completions share the retained event lane: only one of their phases
/// produces work in a round. A provider firehose yields to buffered user input.
#[macro_export]
macro_rules! harness_turn_topology {
 ($backend:ident,$s:ident,$event:ident,$abort:block,$timeout:block,$message:block,$input:block)=>{
  $backend!{
   policy{selection:biased;required_phases:[before_poll];}
   before_poll {tokio::task::yield_now().await;Ok::<(),String>(())}
   #[source(cancel)] #[readiness(quiescent)] #[shutdown]
   _=$s.cancel=>$abort
   #[source(timeout)] #[readiness(quiescent)]
   _=$s.timeout=>$timeout
   #[source(events)] #[readiness(may_remain_ready)] #[yields_to(steer,when=buffered)] #[drain(max=8)]
   $event=$s.events=>$message
   #[source(steer)] #[readiness(may_remain_ready)] #[last]
   $event=$s.steer=>$input
  }
 };
}
pub(crate) struct Context {
    pub turn: TurnId,
    pub generation: u64,
    pub current: Arc<AtomicU64>,
    pub cancel: CancellationToken,
    pub provider: Arc<dyn Provider>,
    pub model: String,
    pub messages: Vec<WireMessage>,
    pub tools: Registry,
    pub cwd: PathBuf,
    pub blobs: Blobs,
    pub redactor: Set,
    pub output: mpsc::Sender<Internal>,
    pub max_rounds: u32,
}
// The bounded lane keeps model events inline to avoid an allocation per streamed token.
#[allow(clippy::large_enum_variant)]
enum Message {
    Model(ModelEvent),
    Tool {
        index: u8,
        output: Box<ToolOutput>,
    },
    ToolDelta {
        item: ItemId,
        chunk: crate::process::Chunk,
    },
}
#[derive(Clone)]
struct Call {
    item: ItemId,
    id: String,
    name: String,
    raw: String,
    started: bool,
    terminal: Option<ItemId>,
    args: Result<Value, String>,
}
struct Sources {
    cancel: source::Cancellation,
    timeout: source::OptionalDeadline,
    events: source::Mpsc<Message, source::close::Dormant>,
    steer: source::Mpsc<(OpId, Vec<InputBlock>), source::close::Dormant>,
}
pub(crate) struct TurnRunner {
    ctx: Context,
    blocks: Vec<InputBlock>,
    rx: Option<mpsc::Receiver<(OpId, Vec<InputBlock>)>>,
    group: TaskGroup,
    tx: mpsc::Sender<Message>,
    events: Option<mpsc::Receiver<Message>>,
    open: BTreeMap<ItemId, (Option<ItemId>, Item)>,
    text: Option<ItemId>,
    reasoning: Option<ItemId>,
    calls: BTreeMap<u8, Call>,
    results: BTreeMap<u8, ToolOutput>,
    finished: std::sync::Arc<std::sync::Mutex<BTreeMap<u8, ToolOutput>>>,
    round: u32,
    pending: VecDeque<(OpId, Vec<InputBlock>)>,
    usage: Option<Usage>,
    call_usages: Vec<Usage>,
}
impl TurnRunner {
    pub fn new(
        ctx: Context,
        blocks: Vec<InputBlock>,
        rx: mpsc::Receiver<(OpId, Vec<InputBlock>)>,
    ) -> Self {
        let group = TaskGroup::new(ctx.cancel.child_token());
        let (tx, events) = mpsc::channel(256);
        Self {
            ctx,
            blocks,
            rx: Some(rx),
            group,
            tx,
            events: Some(events),
            open: BTreeMap::new(),
            text: None,
            reasoning: None,
            calls: BTreeMap::new(),
            results: BTreeMap::new(),
            finished: Default::default(),
            round: 0,
            pending: VecDeque::new(),
            usage: None,
            call_usages: vec![],
        }
    }
    pub async fn run(mut self) {
        let mut sources = Sources {
            cancel: source::cancellation(self.ctx.cancel.clone()),
            timeout: source::OptionalDeadline::new(),
            events: source::mpsc(self.events.take().unwrap(), source::close::Dormant),
            steer: source::mpsc(self.rx.take().unwrap(), source::close::Dormant),
        };
        // A handler panic still reaches group cancellation/join and synthetic pairing.
        let drive = std::panic::AssertUnwindSafe(async {
            self.user(self.blocks.clone(), None).await;
            self.next_round().await?;
            harness_turn_topology!(
                reactor,
                sources,
                event,
                { Ok::<StopReason, String>(StopReason::Cancelled) },
                {
                    Ok(Control::Stop(StopReason::Error {
                        class: ErrorClass::Transient,
                        message: "turn timeout".into(),
                    }))
                },
                { self.message(event).await },
                {
                    self.pending.push_back(event);
                    Ok(Control::Continue)
                }
            )
        })
        .catch_unwind()
        .await;
        let mut reason = match drive {
            Ok(Ok(r)) => r,
            Ok(Err(message)) => StopReason::Error {
                class: ErrorClass::Internal,
                message,
            },
            Err(_) => StopReason::Error {
                class: ErrorClass::Internal,
                message: "turn task panicked".into(),
            },
        };
        if self.group.cancel_and_join().await.is_err() {
            reason = StopReason::Error {
                class: ErrorClass::Internal,
                message: "child task panicked".into(),
            };
        }
        if self.ctx.cancel.is_cancelled() {
            reason = StopReason::Cancelled;
        }
        for (index, result) in self.finished.lock().unwrap().iter() {
            if let Some(call) = self.calls.get(index) {
                if let Some(item) = call.terminal
                    && let Some(Item::Terminal {
                        exit,
                        signal,
                        output,
                        ..
                    }) = &result.terminal
                    && let Some((
                        _,
                        Item::Terminal {
                            exit: e,
                            signal: s,
                            output: b,
                            live,
                            ..
                        },
                    )) = self.open.get_mut(&item)
                {
                    *e = *exit;
                    *s = *signal;
                    b.clone_from(output);
                    *live = None;
                }
                if let Some((_, Item::ToolCall { output, spill, .. })) =
                    self.open.get_mut(&call.item)
                {
                    output.clone_from(&result.output);
                    spill.clone_from(&result.spill);
                }
            }
        }
        let status = if matches!(reason, StopReason::Cancelled) {
            ItemStatus::Cancelled
        } else if matches!(reason, StopReason::Error { .. }) {
            ItemStatus::Failed {
                code: None,
                reason: "turn failed".into(),
            }
        } else {
            ItemStatus::Completed
        };
        while !self.open.is_empty() {
            let id = *self
                .open
                .keys()
                .find(|id| !self.open.values().any(|(p, _)| p.as_ref() == Some(id)))
                .unwrap();
            let (_, mut body) = self.open.remove(&id).unwrap();
            if let Item::ToolCall {
                call_id,
                name,
                input_raw,
                output,
                ..
            } = &mut body
            {
                if call_id.is_empty() {
                    *call_id = id.to_string();
                }
                if name.is_empty() {
                    *name = "interrupted".into();
                }
                if serde_json::from_str::<Value>(input_raw).is_err() {
                    *input_raw = "{}".into();
                }
                if output.is_empty() {
                    output.push(OutputBlock::Error {
                        text: "tool call was not completed".into(),
                    });
                }
            }
            if let Item::Terminal { live, .. } = &mut body {
                *live = None;
            }
            self.emit(
                Event::ItemCompleted {
                    item: id,
                    status: status.clone(),
                    body,
                },
                None,
            )
            .await;
        }
        self.flush_call_usage().await;
        let usage = match Usage::sum_calls(&self.call_usages) {
            Ok(usage) => usage,
            Err(message) => {
                reason = StopReason::Error {
                    class: ErrorClass::Internal,
                    message: message.into(),
                };
                None
            }
        };
        self.emit(
            Event::TurnEnded {
                turn: self.ctx.turn,
                reason,
                usage,
            },
            None,
        )
        .await;
    }
    async fn emit(&self, event: Event, cause: Option<OpId>) {
        let event = self.ctx.redactor.sanitize(&event).unwrap();
        let _ = self
            .ctx
            .output
            .send(Internal::Event {
                generation: self.ctx.generation,
                event,
                cause,
            })
            .await;
    }
    async fn started(&mut self, body: Item, parent: Option<ItemId>, cause: Option<OpId>) -> ItemId {
        let item = ItemId::new();
        self.emit(
            Event::ItemStarted {
                turn: Some(self.ctx.turn),
                item,
                kind: body.kind(),
                parent,
                head: body.head(),
            },
            cause,
        )
        .await;
        self.open.insert(item, (parent, body));
        item
    }
    async fn completed(&mut self, item: ItemId, status: ItemStatus) {
        if let Some((_, body)) = self.open.remove(&item) {
            self.emit(Event::ItemCompleted { item, status, body }, None)
                .await;
        }
    }
    async fn user(&mut self, blocks: Vec<InputBlock>, cause: Option<OpId>) {
        let item = self
            .started(
                Item::UserMessage {
                    blocks: blocks.clone(),
                },
                None,
                cause,
            )
            .await;
        self.completed(item, ItemStatus::Completed).await;
        self.ctx
            .messages
            .push(json!({"role":"user","content":history::input(&blocks)}));
    }
    async fn flush_call_usage(&mut self) {
        let Some(usage) = self.usage.take() else {
            return;
        };
        let usage = usage.authored(
            UsageScope::Call,
            Backend {
                kind: BackendKind::Engine,
                name: self.ctx.provider.id().into(),
                version: Some(env!("CARGO_PKG_VERSION").into()),
            },
            Some(self.ctx.model.clone()),
        );
        if !usage.has_reported_numbers() {
            return;
        }
        self.call_usages.push(usage.clone());
        let item = self.started(Item::Usage { usage }, None, None).await;
        self.completed(item, ItemStatus::Completed).await;
    }
    async fn next_round(&mut self) -> Result<(), String> {
        if self.round >= self.ctx.max_rounds {
            return Err("round limit reached".into());
        }
        self.round += 1;
        while let Some((op, blocks)) = self.pending.pop_front() {
            self.user(blocks, Some(op)).await;
        }
        self.calls.clear();
        self.results.clear();
        self.finished.lock().unwrap().clear();
        self.text = None;
        self.reasoning = None;
        let tools = self
            .ctx
            .tools
            .definitions()
            .iter()
            .map(|d| d.wire.clone())
            .collect::<Vec<_>>();
        let (reply, rx) = oneshot::channel();
        if !send(
            &self.ctx.output,
            Internal::PrepareRound {
                generation: self.ctx.generation,
                turn: self.ctx.turn,
                tools: tools.clone(),
                reply,
            },
            &self.group.cancel,
        )
        .await
        {
            return Err("cancelled".into());
        }
        let (messages, stable_prefix) = tokio::select! {biased;_=self.group.cancel.cancelled()=>return Err("cancelled".into()),value=rx=>value.map_err(|_|"context actor closed")?.map_err(|_|"context replay failed")?};
        history::check_wire(&messages)?;
        let req = ModelRequest {
            model: self.ctx.model.clone(),
            messages,
            tools,
            tool_choice: ToolChoice::Auto,
            max_tokens: None,
            stable_prefix,
        };
        let provider = self.ctx.provider.clone();
        let cancel = self.group.cancel.child_token();
        let tx = self.tx.clone();
        self.group.spawn(async move {
            let task = std::panic::AssertUnwindSafe(async {
                let mut stream = provider.stream(req, cancel.clone());
                loop {
                    let e = tokio::select! {biased;_=cancel.cancelled()=>break,e=stream.next()=>e};
                    let Some(e) = e else {
                        break;
                    };
                    let end = matches!(e, ModelEvent::Finish(_) | ModelEvent::Failed(_));
                    if !send(&tx, Message::Model(e), &cancel).await || end {
                        break;
                    }
                }
            })
            .catch_unwind()
            .await;
            if task.is_err() {
                let _ = send(
                    &tx,
                    Message::Model(ModelEvent::Failed(crate::provider::ProviderError::new(
                        ErrorClass::Internal,
                        "provider task panicked",
                    ))),
                    &cancel,
                )
                .await;
            }
        });
        Ok(())
    }
    async fn message(&mut self, event: Message) -> Result<Control<StopReason>, String> {
        match event {
            Message::Model(ModelEvent::Text(text)) => {
                let id = if let Some(id) = self.text {
                    id
                } else {
                    let id = self
                        .started(
                            Item::AgentMessage {
                                markdown: String::new(),
                            },
                            None,
                            None,
                        )
                        .await;
                    self.text = Some(id);
                    id
                };
                let delta = Delta::Text { append: text };
                self.open.get_mut(&id).unwrap().1.apply(&delta);
                self.emit(Event::ItemDelta { item: id, delta }, None).await;
            }
            Message::Model(ModelEvent::Reasoning(text)) => {
                let id = self.reasoning_id().await;
                let delta = Delta::Reasoning { append: text };
                self.open.get_mut(&id).unwrap().1.apply(&delta);
                self.emit(Event::ItemDelta { item: id, delta }, None).await;
            }
            Message::Model(ModelEvent::ReasoningOpaque(value)) => {
                let id = self.reasoning_id().await;
                if let Item::Reasoning {
                    provider_opaque, ..
                } = &mut self.open.get_mut(&id).unwrap().1
                {
                    *provider_opaque = Some(
                        json!({"provider":self.ctx.provider.id(),"form":"native","data":value}),
                    );
                }
            }
            Message::Model(ModelEvent::ToolCallDelta {
                index,
                id,
                name,
                args,
            }) => {
                let call = self.calls.entry(index).or_insert_with(|| Call {
                    item: ItemId::new(),
                    id: String::new(),
                    name: String::new(),
                    raw: String::new(),
                    started: false,
                    terminal: None,
                    args: Err("incomplete arguments".into()),
                });
                if let Some(id) = id {
                    call.id = id;
                }
                if let Some(name) = name {
                    call.name = name;
                }
                call.raw.push_str(&args);
                let ready = !call.id.is_empty() && !call.name.is_empty();
                let started = call.started;
                let item = call.item;
                let body = Item::ToolCall {
                    title: None,
                    server: None,
                    raw_output: None,
                    call_id: call.id.clone(),
                    name: call.name.clone(),
                    tool_kind: self
                        .ctx
                        .tools
                        .get(&call.name)
                        .map(|d| d.kind.clone())
                        .unwrap_or(ToolKind::Other),
                    input_raw: call.raw.clone(),
                    output: vec![],
                    locations: vec![],
                    spill: None,
                };
                let append = if started { args } else { call.raw.clone() };
                if ready {
                    if !started {
                        call.started = true;
                        self.emit(
                            Event::ItemStarted {
                                turn: Some(self.ctx.turn),
                                item,
                                kind: ItemKind::ToolCall,
                                parent: None,
                                head: body.head(),
                            },
                            None,
                        )
                        .await;
                    }
                    self.open.insert(item, (None, body));
                    self.emit(
                        Event::ItemDelta {
                            item,
                            delta: Delta::ToolInput { append },
                        },
                        None,
                    )
                    .await;
                }
            }
            Message::Model(ModelEvent::ToolCallReady {
                index,
                id,
                name,
                args_raw,
                args,
            }) => {
                let Some(c) = self.calls.get_mut(&index) else {
                    return Err("ready without tool delta".into());
                };
                c.id = id;
                c.name = name;
                c.raw = args_raw;
                c.args = args;
                if let Item::ToolCall {
                    call_id,
                    name,
                    tool_kind,
                    input_raw,
                    ..
                } = &mut self.open.get_mut(&c.item).unwrap().1
                {
                    call_id.clone_from(&c.id);
                    name.clone_from(&c.name);
                    *tool_kind = self
                        .ctx
                        .tools
                        .get(&c.name)
                        .map(|d| d.kind.clone())
                        .unwrap_or(ToolKind::Other);
                    input_raw.clone_from(&c.raw);
                }
            }
            Message::Model(ModelEvent::Usage(usage)) => {
                self.usage = Some(usage);
            }
            Message::Model(ModelEvent::Retrying {
                attempt,
                wait_ms,
                class,
            }) => {
                self.flush_call_usage().await;
                let id = self
                    .started(
                        Item::Notice {
                            level: Level::Warn,
                            text: format!("retry {attempt} in {wait_ms} ms ({class:?})"),
                        },
                        None,
                        None,
                    )
                    .await;
                self.completed(id, ItemStatus::Completed).await;
                self.emit(
                    Event::Status {
                        phase: Phase::Retrying { attempt, wait_ms },
                    },
                    None,
                )
                .await;
            }
            Message::Model(ModelEvent::Failed(error)) => {
                return Ok(Control::Stop(if error.class == ErrorClass::Cancelled {
                    StopReason::Cancelled
                } else {
                    StopReason::Error {
                        class: error.class,
                        message: error.message,
                    }
                }));
            }
            Message::Model(ModelEvent::Finish(finish)) => {
                self.flush_call_usage().await;
                self.save_assistant(matches!(finish, Finish::ToolCalls))
                    .await;
                if matches!(finish, Finish::ToolCalls) {
                    for (index, call) in self.calls.clone() {
                        self.spawn_tool(index, call).await;
                    }
                    if self.calls.is_empty() {
                        return Err("tool finish without calls".into());
                    }
                } else {
                    return Ok(Control::Stop(match finish {
                        Finish::Stop => StopReason::EndTurn,
                        Finish::Length => StopReason::MaxTokens,
                        Finish::ContentFilter => StopReason::Error {
                            class: ErrorClass::BadRequest,
                            message: "content filtered".into(),
                        },
                        Finish::ToolCalls => unreachable!(),
                    }));
                }
            }
            Message::ToolDelta { item, chunk } => {
                if let Some((_, body)) = self.open.get_mut(&item) {
                    let delta = Delta::Output {
                        stream: chunk.stream,
                        append: chunk.text,
                    };
                    body.apply(&delta);
                    self.emit(Event::ItemDelta { item, delta }, None).await;
                }
            }
            Message::Tool { index, output } => {
                if self.ctx.current.load(Ordering::Acquire) != self.ctx.generation {
                    return Ok(Control::Continue);
                }
                let Some(call) = self.calls.get(&index).cloned() else {
                    return Err("orphan tool completion".into());
                };
                for body in output.changes.iter().cloned() {
                    let item = self.started(body, Some(call.item), None).await;
                    self.completed(item, ItemStatus::Completed).await;
                }
                if let Some(item) = call.terminal {
                    if let Some(Item::Terminal {
                        exit,
                        signal,
                        output: blob,
                        ..
                    }) = output.terminal.as_ref()
                        && let Some((
                            _,
                            Item::Terminal {
                                exit: e,
                                signal: s,
                                output: b,
                                live,
                                ..
                            },
                        )) = self.open.get_mut(&item)
                    {
                        *e = *exit;
                        *s = *signal;
                        b.clone_from(blob);
                        *live = None;
                    }
                    if let Some((_, Item::Terminal { live, .. })) = self.open.get_mut(&item) {
                        *live = None;
                    }
                    self.completed(
                        item,
                        if output.failed {
                            ItemStatus::Failed {
                                code: None,
                                reason: output
                                    .output
                                    .iter()
                                    .find_map(|b| match b {
                                        OutputBlock::Text { text }
                                        | OutputBlock::Error { text } => {
                                            text.lines().next().map(str::to_string)
                                        }
                                        _ => None,
                                    })
                                    .unwrap_or_else(|| "tool failed".into()),
                            }
                        } else {
                            ItemStatus::Completed
                        },
                    )
                    .await;
                }
                if let Item::ToolCall {
                    output: blocks,
                    spill,
                    ..
                } = &mut self.open.get_mut(&call.item).unwrap().1
                {
                    blocks.clone_from(&output.output);
                    spill.clone_from(&output.spill);
                }
                self.completed(
                    call.item,
                    if output.failed {
                        ItemStatus::Failed {
                            code: None,
                            reason: output
                                .output
                                .iter()
                                .find_map(|b| match b {
                                    OutputBlock::Text { text } | OutputBlock::Error { text } => {
                                        text.lines().next().map(str::to_string)
                                    }
                                    _ => None,
                                })
                                .unwrap_or_else(|| "tool failed".into()),
                        }
                    } else {
                        ItemStatus::Completed
                    },
                )
                .await;
                self.results.insert(index, *output);
                if self.results.len() == self.calls.len() {
                    for (&index, call) in &self.calls {
                        self.ctx.messages.push(json!({"role":"tool","tool_call_id":call.id,"content":history::output_text(&self.results[&index].output)}));
                    }
                    if self.round >= self.ctx.max_rounds {
                        return Ok(Control::Stop(StopReason::MaxRounds));
                    }
                    self.group.reap().await;
                    self.next_round().await?;
                }
            }
        }
        Ok(Control::Continue)
    }
    async fn reasoning_id(&mut self) -> ItemId {
        if let Some(id) = self.reasoning {
            id
        } else {
            let id = self
                .started(
                    Item::Reasoning {
                        summary: String::new(),
                        raw: None,
                        provider_opaque: None,
                    },
                    None,
                    None,
                )
                .await;
            self.reasoning = Some(id);
            id
        }
    }
    async fn save_assistant(&mut self, tools: bool) {
        let text = self
            .text
            .and_then(|id| self.open.get(&id))
            .and_then(|(_, b)| {
                if let Item::AgentMessage { markdown } = b {
                    Some(markdown.clone())
                } else {
                    None
                }
            })
            .unwrap_or_default();
        let mut wire = json!({"role":"assistant","content":text});
        if let Some((
            _,
            Item::Reasoning {
                summary,
                provider_opaque,
                ..
            },
        )) = self.reasoning.and_then(|id| self.open.get(&id))
        {
            wire["reasoning"] = json!(summary);
            if let Some(v) = provider_opaque {
                wire["reasoning_details"] = v.clone();
            }
        }
        if tools {
            wire["tool_calls"]=json!(self.calls.values().map(|c|json!({"id":c.id,"type":"function","function":{"name":c.name,"arguments":c.raw}})).collect::<Vec<_>>());
        }
        self.ctx.messages.push(wire);
        if let Some(id) = self.text.take() {
            self.completed(id, ItemStatus::Completed).await;
        }
        if let Some(id) = self.reasoning.take() {
            self.completed(id, ItemStatus::Completed).await;
        }
    }
    async fn spawn_tool(&mut self, index: u8, call: Call) {
        let terminal = if let Ok(args) = call.args.as_ref()
            && matches!(call.name.as_str(), "exec" | "bash")
        {
            let command = if call.name == "bash" {
                format!("bash -c {}", args["command"].as_str().unwrap_or(""))
            } else {
                format!(
                    "{} {}",
                    args["program"].as_str().unwrap_or(""),
                    args["args"]
                        .as_array()
                        .map(|a| a
                            .iter()
                            .filter_map(|s| s.as_str())
                            .collect::<Vec<_>>()
                            .join(" "))
                        .unwrap_or_default()
                )
            };
            let body = Item::Terminal {
                terminal: ItemId::new().to_string(),
                command: self.ctx.redactor.scrub(&command),
                exit: None,
                signal: None,
                output: None,
                live: Some(String::new()),
            };
            let item = self.started(body, Some(call.item), None).await;
            self.calls.get_mut(&index).unwrap().terminal = Some(item);
            Some(item)
        } else {
            None
        };
        let registry = self.ctx.tools.clone();
        let context = crate::tools::Context {
            cwd: self.ctx.cwd.clone(),
            blobs: self.ctx.blobs.clone(),
            redactor: self.ctx.redactor.clone(),
            generation: self.ctx.generation,
            current_generation: self.ctx.current.clone(),
            cancel: self.group.cancel.child_token(),
        };
        let cancel = context.cancel.clone();
        let actor = self.ctx.output.clone();
        let tx = self.tx.clone();
        let turn = self.ctx.turn;
        let generation = self.ctx.generation;
        let finished = self.finished.clone();
        self.group.spawn(async move{
   let result=std::panic::AssertUnwindSafe(async {
    let args=match call.args{Ok(v)=>v,Err(e)=>return ToolOutput::error(e)};let Some(def)=registry.get(&call.name)else{return ToolOutput::error("unknown tool");};
    let permit=if let Some(effect)=def.effect.effect(){let item=ItemId::new();let (reply,wait)=oneshot::channel();let paths=args["path"].as_str().map(|p|vec![PathBuf::from(p)]).unwrap_or_default();let body=Item::Approval{subject:ApprovalSubject{tool_call:call.item,effect,summary:format!("{} {}",call.name,call.raw),paths,command:args["command"].as_str().map(str::to_owned)},options:vec![ApprovalDecisionKind::Allow,ApprovalDecisionKind::AllowAlways,ApprovalDecisionKind::Deny],decision:None,by:None};
      let request=Internal::Approval{generation,turn,item,tool:call.name.clone(),effect:def.effect.clone(),body,reply};if !send(&actor,request,&cancel).await{return ToolOutput::error("cancelled");}
      tokio::select!{biased;_=cancel.cancelled()=>return ToolOutput::error("cancelled"),r=wait=>match r{Ok(Ok(p))=>Some(p),Ok(Err(reason))=>return ToolOutput::error(reason),Err(_)=>return ToolOutput::error("cancelled")}}
    }else{None};
    let (out,mut chunks)=mpsc::channel(32);
    let execution=registry.run_stream(&call.name,args,context,permit,terminal.map(|_|out));tokio::pin!(execution);let mut open=terminal.is_some();
    loop{tokio::select!{biased;
      chunk=chunks.recv(),if open=>match chunk{Some(chunk)=>{let _=send(&tx,Message::ToolDelta{item:terminal.unwrap(),chunk},&cancel).await;},None=>open=false},
      result=&mut execution=>{while let Ok(chunk)=chunks.try_recv(){let _=send(&tx,Message::ToolDelta{item:terminal.unwrap(),chunk},&cancel).await;}break result;}
    }}
   }).catch_unwind().await.unwrap_or_else(|_|ToolOutput::error("tool panicked"));
   finished.lock().unwrap().insert(index,result.clone());
   let _=send(&tx,Message::Tool{index,output:Box::new(result)},&cancel).await;
  });
    }
}
