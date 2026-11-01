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
enum Message {
    Model(ModelEvent),
    Tool { index: u8, output: Box<ToolOutput> },
}
#[derive(Clone)]
struct Call {
    item: ItemId,
    id: String,
    name: String,
    raw: String,
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
    round: u32,
    pending: VecDeque<(OpId, Vec<InputBlock>)>,
    usage: Option<Usage>,
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
            round: 0,
            pending: VecDeque::new(),
            usage: None,
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
        let status = if matches!(reason, StopReason::Cancelled) {
            ItemStatus::Cancelled
        } else if matches!(reason, StopReason::Error { .. }) {
            ItemStatus::Failed {
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
        let usage = self.usage.take();
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
        self.text = None;
        self.reasoning = None;
        let prefix = json!({"role":"system","content":format!("You are bboxy450-harness, a coding assistant. Use the provided tools. Working directory: {}",self.ctx.cwd.display())});
        let mut messages = vec![prefix];
        messages.extend(self.ctx.messages.clone());
        history::check_wire(&messages)?;
        let req = ModelRequest {
            model: self.ctx.model.clone(),
            messages,
            tools: self
                .ctx
                .tools
                .definitions()
                .iter()
                .map(|d| d.wire.clone())
                .collect(),
            tool_choice: ToolChoice::Auto,
            max_tokens: None,
            stable_prefix: 1,
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
                    *provider_opaque = Some(value);
                }
            }
            Message::Model(ModelEvent::ToolCallDelta {
                index,
                id,
                name,
                args,
            }) => {
                if !self.calls.contains_key(&index) {
                    let item = self
                        .started(
                            Item::ToolCall {
                                call_id: id.clone().unwrap_or_default(),
                                name: name.clone().unwrap_or_default(),
                                tool_kind: ToolKind::Other,
                                input_raw: String::new(),
                                output: vec![],
                                locations: vec![],
                                spill: None,
                            },
                            None,
                            None,
                        )
                        .await;
                    self.calls.insert(
                        index,
                        Call {
                            item,
                            id: String::new(),
                            name: String::new(),
                            raw: String::new(),
                            args: Err("incomplete arguments".into()),
                        },
                    );
                }
                let call = self.calls.get_mut(&index).unwrap();
                if let Some(id) = id {
                    call.id = id;
                }
                if let Some(name) = name {
                    call.name = name;
                }
                call.raw.push_str(&args);
                let item = call.item;
                if let Item::ToolCall {
                    call_id,
                    name,
                    input_raw,
                    ..
                } = &mut self.open.get_mut(&item).unwrap().1
                {
                    call_id.clone_from(&call.id);
                    name.clone_from(&call.name);
                    input_raw.clone_from(&call.raw);
                }
                self.emit(
                    Event::ItemDelta {
                        item,
                        delta: Delta::ToolInput { append: args },
                    },
                    None,
                )
                .await;
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
                        .map(|d| d.kind)
                        .unwrap_or(ToolKind::Other);
                    input_raw.clone_from(&c.raw);
                }
            }
            Message::Model(ModelEvent::Usage(usage)) => self.usage = Some(usage),
            Message::Model(ModelEvent::Retrying {
                attempt,
                wait_ms,
                class,
            }) => {
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
                self.save_assistant(matches!(finish, Finish::ToolCalls))
                    .await;
                if matches!(finish, Finish::ToolCalls) {
                    for (index, call) in self.calls.clone() {
                        self.spawn_tool(index, call);
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
            Message::Tool { index, output } => {
                if self.ctx.current.load(Ordering::Acquire) != self.ctx.generation {
                    return Ok(Control::Continue);
                }
                let Some(call) = self.calls.get(&index).cloned() else {
                    return Err("orphan tool completion".into());
                };
                for body in output
                    .changes
                    .iter()
                    .cloned()
                    .chain(output.terminal.iter().cloned())
                {
                    let item = self.started(body, Some(call.item), None).await;
                    self.completed(item, ItemStatus::Completed).await;
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
                            reason: "tool failed".into(),
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
    fn spawn_tool(&mut self, index: u8, call: Call) {
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
        self.group.spawn(async move{
   let result=std::panic::AssertUnwindSafe(async {
    let args=match call.args{Ok(v)=>v,Err(e)=>return ToolOutput::error(e)};let Some(def)=registry.get(&call.name)else{return ToolOutput::error("unknown tool");};
    let permit=if let Some(effect)=def.effect.effect(){let item=ItemId::new();let (reply,wait)=oneshot::channel();let paths=args["path"].as_str().map(|p|vec![PathBuf::from(p)]).unwrap_or_default();let body=Item::Approval{subject:ApprovalSubject{tool_call:call.item,effect,summary:format!("{} {}",call.name,call.raw),paths,command:args["command"].as_str().map(str::to_owned)},options:vec![ApprovalDecisionKind::Allow,ApprovalDecisionKind::AllowAlways,ApprovalDecisionKind::Deny],decision:None,by:None};
      let request=Internal::Approval{generation,turn,item,tool:call.name.clone(),effect:def.effect.clone(),body,reply};if !send(&actor,request,&cancel).await{return ToolOutput::error("cancelled");}
      tokio::select!{biased;_=cancel.cancelled()=>return ToolOutput::error("cancelled"),r=wait=>match r{Ok(Some(p))=>Some(p),_=>return ToolOutput::error("policy denied this tool")}}
    }else{None};
    registry.run(&call.name,args,context,permit).await
   }).catch_unwind().await.unwrap_or_else(|_|ToolOutput::error("tool panicked"));
   let _=send(&tx,Message::Tool{index,output:Box::new(result)},&cancel).await;
  });
    }
}
