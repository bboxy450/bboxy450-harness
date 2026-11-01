// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    engine::{Config, SessionActor},
    gateway::{Gateway, Subscribe},
    policy::Policy,
    protocol::*,
    provider::{
        self, ModelEvent, ModelRequest, Provider, ProviderError,
        profile::{Profile, Unattended},
    },
    redact::Set,
};
use std::{path::PathBuf, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
#[derive(Clone)]
struct Failed(ProviderError);
impl Provider for Failed {
    fn caps(&self) -> provider::Caps {
        provider::Caps {
            images: false,
            reasoning: false,
            tool_choice: false,
            context_tokens: 0,
        }
    }
    fn stream(
        &self,
        _req: ModelRequest,
        _cancel: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = ModelEvent> + Send>> {
        let e = self.0.clone();
        Box::pin(futures_util::stream::once(
            async move { ModelEvent::Failed(e) },
        ))
    }
}
pub async fn cli() -> i32 {
    match run_cli().await {
        Ok(code) => code,
        Err(message) => {
            eprintln!("bboxy450-harness: {message}");
            1
        }
    }
}
async fn run_cli() -> Result<i32, String> {
    let mut args = std::env::args().skip(1);
    let mut prompt = None;
    let mut json = false;
    let mut seat = false;
    let mut profile = "nemotron".to_owned();
    let mut cwd = std::env::current_dir().map_err(|_| "working directory unavailable")?;
    let mut sessions = None;
    let mut scripts = vec![];
    let mut resume = None;
    let mut yolo = false;
    let mut max_rounds = 50;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "smoke-live" => return Ok(super::smoke::run().await),
            "seat" => seat = true,
            "-p" | "--prompt" => prompt = Some(args.next().ok_or("missing prompt")?),
            "--json" => json = true,
            "--profile" => profile = args.next().ok_or("missing profile")?,
            "--cwd" => cwd = PathBuf::from(args.next().ok_or("missing cwd")?),
            "--sessions" => {
                sessions = Some(PathBuf::from(args.next().ok_or("missing sessions path")?))
            }
            "--scripted" => scripts.push(PathBuf::from(args.next().ok_or("missing SSE path")?)),
            "--resume" => resume = Some(args.next().ok_or("missing session")?),
            "--yolo" => yolo = true,
            "--max-rounds" => {
                max_rounds = args
                    .next()
                    .ok_or("missing max rounds")?
                    .parse()
                    .map_err(|_| "invalid max rounds")?
            }
            "--export-schema" => {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&crate::protocol::schema()).unwrap()
                );
                return Ok(0);
            }
            "check-transcript" => {
                let path = args.next().ok_or("missing transcript path")?;
                let text = tokio::fs::read_to_string(path)
                    .await
                    .map_err(|_| "transcript unavailable")?;
                let events = text
                    .lines()
                    .map(serde_json::from_str)
                    .collect::<Result<Vec<Envelope>, _>>()
                    .map_err(|_| "invalid transcript JSON")?;
                crate::protocol::laws::check_transcript(&events).map_err(|e| e.to_string())?;
                return Ok(0);
            }
            "--help" | "-h" => {
                println!(
                    "bboxy450-harness -p PROMPT --json [--profile NAME] [--cwd DIR]\nbboxy450-harness seat [--resume SESSION] [--cwd DIR]\n--scripted SSE (repeat for rounds) --sessions DIR --yolo"
                );
                return Ok(0);
            }
            _ => return Err(format!("unknown option {arg}")),
        }
    }
    if !seat && prompt.is_none() {
        return Err("provide -p PROMPT --json, or seat".into());
    }
    let root = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
        .join(".bboxy450-harness");
    let sessions = sessions.unwrap_or_else(|| root.join("sessions"));
    let redactor = Set::default();
    if let Ok(value) = std::env::var("BBOXY_CAPABILITY") {
        redactor.register(&value);
    }
    let (provider, model, mut policy): (Arc<dyn Provider>, String, Policy) = if !scripts.is_empty()
    {
        let mut rounds = vec![];
        for path in scripts {
            rounds.push(
                tokio::fs::read(path)
                    .await
                    .map_err(|_| "scripted SSE unavailable")?,
            );
        }
        (
            Arc::new(provider::ScriptedProvider::new(rounds)),
            "scripted".into(),
            Policy::new(Unattended::Deny),
        )
    } else {
        let p = Profile::path(&profile).map_err(|e| e.message)?;
        match Profile::load(&p).await {
            Ok(p) => match p.key(&redactor, CancellationToken::new()).await {
                Ok(key) => {
                    let options = provider::Options {
                        retry_deadline: p.retry_deadline.map(std::time::Duration::from_secs),
                        ..Default::default()
                    };
                    let client =
                        provider::OpenAiCompat::new(&p.endpoint, key, redactor.clone(), options)
                            .map_err(|e| e.message)?;
                    (Arc::new(client), p.model.clone(), p.policy())
                }
                Err(e) => (
                    Arc::new(Failed(ProviderError::new(ErrorClass::Config, e.message))),
                    "unconfigured".into(),
                    Policy::new(Unattended::Deny),
                ),
            },
            Err(e) => (
                Arc::new(Failed(ProviderError::new(ErrorClass::Config, e.message))),
                "unconfigured".into(),
                Policy::new(Unattended::Deny),
            ),
        }
    };
    if yolo {
        policy = Policy::allow_all(Unattended::Deny);
    }
    let session = if let Some(id) = resume {
        SessionId(id.parse().map_err(|_| "invalid session id")?)
    } else {
        SessionId::new()
    };
    let config = Config {
        meta: SessionMeta {
            session,
            cwd,
            profile: Some(profile),
            fork: None,
            protocol: PROTOCOL,
            harness: env!("CARGO_PKG_VERSION").into(),
            created_ms: crate::store::now_ms(),
        },
        sessions,
        provider,
        profile_error: (model == "unconfigured").then_some("profile could not be resolved".into()),
        model,
        policy,
        redactor,
        max_rounds,
    };
    let (gateway, task) = SessionActor::open(config, OpId::new())
        .await
        .map_err(|_| "session could not be opened")?;
    let socket = crate::gateway::unix::UnixServer::bind(
        root.join("run").join(format!("{session}.sock")),
        gateway.clone(),
    )
    .await
    .map_err(|_| "session socket unavailable")?;
    let mut subscription = gateway
        .subscribe(Subscribe {
            session,
            after: None,
            ephemeral: false,
        })
        .map_err(|_| "subscribe refused")?;
    let mut stdout = tokio::io::stdout();
    let mut input = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    let mut hup = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup())
        .map_err(|_| "signal handler failed")?;
    let mut term = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
        .map_err(|_| "signal handler failed")?;
    let mut stopping = false;
    let mut running = false;
    let mut failed = false;
    let op = |op| OpEnvelope {
        v: PROTOCOL,
        session,
        op_id: OpId::new(),
        op,
    };
    if let Some(prompt) = prompt {
        gateway
            .send(op(Op::UserTurn {
                blocks: vec![InputBlock::Text { text: prompt }],
                mode: TurnMode::Queue,
            }))
            .map_err(|_| "turn refused")?;
    }
    loop {
        tokio::select! {biased;
         _=hup.recv()=>{let _=gateway.send(op(Op::Close));stopping=true;},
         _=term.recv()=>{let _=gateway.send(op(Op::Close));stopping=true;},
         _=tokio::signal::ctrl_c()=>{if running{let _=gateway.send(op(Op::Interrupt{turn:None}));}},
         line=input.next_line(),if seat&&!stopping=>{match line{Ok(Some(text))=>{if !text.is_empty(){let _=gateway.send(op(Op::UserTurn{blocks:vec![InputBlock::Text{text}],mode:TurnMode::Queue}));}},_=>{let _=gateway.send(op(Op::Close));stopping=true;}}},
         event=subscription.recv()=>{let Some(event)=event else{break;};match &event.event{Event::TurnStarted{..}=>running=true,Event::TurnEnded{reason,..}=>{running=false;failed|=matches!(reason,StopReason::Error{..});if !seat&&!stopping{let _=gateway.send(op(Op::Close));stopping=true;}},_=>{}}
          let bytes=if json {let mut b=serde_json::to_vec(&event).map_err(|_|"event serialization failed")?;b.push(b'\n');b}else{super::seat::render(&event)};
          if stdout.write_all(&bytes).await.is_err(){let _=gateway.send(op(Op::Close));break;}let _=stdout.flush().await;
          if matches!(event.event,Event::SessionClosed){break;}
         }
        }
    }
    socket.close().await.map_err(|_| "socket shutdown failed")?;
    task.await
        .map_err(|_| "actor task failed")?
        .map_err(|_| "session store failed")?;
    Ok(if failed { 1 } else { 0 })
}
