// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    engine::Config,
    gateway::{Gateway, Host, Subscribe},
    policy::Policy,
    protocol::*,
    provider::{
        self, FailedProvider, Provider, ProviderError,
        profile::{Profile, Unattended},
    },
    redact::Set,
};
use std::{path::PathBuf, sync::Arc};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
pub async fn cli() -> i32 {
    let result=async {
        let stop=CancellationToken::new();let ready=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut hup=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()).map_err(|_|"signal handler failed".to_string())?;
        let mut term=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).map_err(|_|"signal handler failed".to_string())?;
        let mut interrupt=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).map_err(|_|"signal handler failed".to_string())?;
        let run=run_cli(stop.clone(),ready.clone());tokio::pin!(run);
        loop {tokio::select!{biased;
            result=&mut run=>break result,
            _=hup.recv()=>stop.cancel(),
            _=term.recv()=>stop.cancel(),
            _=interrupt.recv(),if !ready.load(std::sync::atomic::Ordering::Acquire)=>stop.cancel(),
        }}
    }.await;
    match result {
        Ok(code) => code,
        Err(message) => {
            eprintln!("bboxy450-harness: {message}");
            1
        }
    }
}
async fn run_cli(
    stop: CancellationToken,
    ready: Arc<std::sync::atomic::AtomicBool>,
) -> Result<i32, String> {
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
                    "bboxy450-harness -p PROMPT --json [--profile NAME] [--cwd DIR]\nbboxy450-harness --json --resume CLOSED_SESSION\nbboxy450-harness seat [--resume SESSION] [--cwd DIR]\n--scripted SSE (repeat for rounds) --sessions DIR --yolo"
                );
                return Ok(0);
            }
            _ => return Err(format!("unknown option {arg}")),
        }
    }
    if !seat && prompt.is_none() && !(json && resume.is_some()) {
        return Err("provide -p PROMPT --json, or seat".into());
    }
    let root = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
        .join(".bboxy450-harness");
    let sessions = sessions.unwrap_or_else(|| root.join("sessions"));
    if !seat && prompt.is_none() {
        let session = SessionId(
            resume
                .as_ref()
                .ok_or("missing session")?
                .parse()
                .map_err(|_| "invalid session id")?,
        );
        let events = crate::store::Store::replay_closed(sessions, session)
            .await
            .map_err(|_| "closed session replay unavailable")?;
        let mut stdout = tokio::io::stdout();
        for event in events {
            let mut bytes = serde_json::to_vec(&event).map_err(|_| "event serialization failed")?;
            bytes.push(b'\n');
            stdout
                .write_all(&bytes)
                .await
                .map_err(|_| "output unavailable")?;
        }
        stdout.flush().await.map_err(|_| "output unavailable")?;
        return Ok(0);
    }
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
        let loaded =
            tokio::select! {biased;_=stop.cancelled()=>return Ok(0),p=Profile::load(&p)=>p};
        match loaded {
            Ok(p) => match p.key(&redactor, stop.child_token()).await {
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
                    Arc::new(FailedProvider(ProviderError::new(
                        ErrorClass::Config,
                        e.message,
                    ))),
                    "unconfigured".into(),
                    Policy::new(Unattended::Deny),
                ),
            },
            Err(e) => (
                Arc::new(FailedProvider(ProviderError::new(
                    ErrorClass::Config,
                    e.message,
                ))),
                "unconfigured".into(),
                Policy::new(Unattended::Deny),
            ),
        }
    };
    if stop.is_cancelled() {
        return Ok(0);
    }
    if yolo {
        policy = Policy::allow_all(Unattended::Deny);
    }
    let session = if let Some(id) = &resume {
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
        profiles: root.join("profiles"),
        provider,
        profile_error: (model == "unconfigured").then_some("profile could not be resolved".into()),
        model,
        policy,
        redactor,
        max_rounds,
    };
    let start = OpEnvelope {
        v: PROTOCOL,
        session,
        op_id: OpId::new(),
        op: Op::StartSession {
            cwd: config.meta.cwd.clone(),
            profile: config.meta.profile.clone(),
            fork: config.meta.fork,
        },
    };
    let host = Host::new(config);
    let gateway = if resume.is_some() {
        host.open(session)
            .await
            .map_err(|_| "session could not be opened")?
    } else {
        host.create(start)
            .await
            .map_err(|_| "session could not be created")?
            .0
    };
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
            ephemeral: seat && !json,
        })
        .map_err(|_| "subscribe refused")?;
    let mut line_projection = super::seat::LineProjection::default();
    let mut stdout = tokio::io::stdout();
    let mut input = tokio::io::BufReader::new(tokio::io::stdin()).lines();
    ready.store(true, std::sync::atomic::Ordering::Release);
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
         _=stop.cancelled(),if !stopping=>{let _=gateway.send(op(Op::Close));stopping=true;},
         _=tokio::signal::ctrl_c()=>{if running{let _=gateway.send(op(Op::Interrupt{turn:None}));}},
         line=input.next_line(),if seat&&!stopping=>{match line{Ok(Some(text))=>{if !text.is_empty(){let _=gateway.send(op(Op::UserTurn{blocks:vec![InputBlock::Text{text}],mode:TurnMode::Queue}));}},_=>{let _=gateway.send(op(Op::Close));stopping=true;}}},
         event=subscription.recv()=>{let Some(event)=event else{break;};match &event.event{Event::TurnStarted{..}=>running=true,Event::TurnEnded{reason,..}=>{running=false;failed|=matches!(reason,StopReason::Error{..});if !seat&&!stopping{let _=gateway.send(op(Op::Close));stopping=true;}},_=>{}}
          let bytes=if json {let mut b=serde_json::to_vec(&event).map_err(|_|"event serialization failed")?;b.push(b'\n');b}else{line_projection.render(&event)};
          if stdout.write_all(&bytes).await.is_err(){let _=gateway.send(op(Op::Close));break;}let _=stdout.flush().await;
          if matches!(event.event,Event::SessionClosed){break;}
         }
        }
    }
    socket.close().await.map_err(|_| "socket shutdown failed")?;
    host.join().await.map_err(|_| "session actor failed")?;
    Ok(if failed { 1 } else { 0 })
}
