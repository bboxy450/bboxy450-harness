// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use crate::{
    engine::{Config, LaunchOptions},
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
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
// Preserve I/O errors as teardown observations; cleanup must not depend on
// a later snapshot of terminal readiness.
fn stop_cli(stop: &CancellationToken) {
    crate::process::kill_owned_groups();
    stop.cancel();
}
pub async fn cli() -> i32 {
    let result=async {
        crate::process::install_shutdown_handlers().map_err(|_|"signal handler failed".to_string())?;
        let stop=CancellationToken::new();let ready=Arc::new(std::sync::atomic::AtomicBool::new(false));
        let mut hup=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::hangup()).map_err(|_|"signal handler failed".to_string())?;
        let mut term=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()).map_err(|_|"signal handler failed".to_string())?;
        let mut interrupt=tokio::signal::unix::signal(tokio::signal::unix::SignalKind::interrupt()).map_err(|_|"signal handler failed".to_string())?;
        let tty=unsafe {libc::isatty(libc::STDIN_FILENO)}==1;
        let mut shutdown_watch=tokio::time::interval(std::time::Duration::from_millis(5));
        shutdown_watch.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let run=run_cli(stop.clone(),ready.clone());tokio::pin!(run);
        loop {tokio::select!{biased;
            result=&mut run=>break result,
            _=hup.recv()=>stop.cancel(),
            _=term.recv()=>stop.cancel(),
            _=interrupt.recv(),if !ready.load(std::sync::atomic::Ordering::Acquire)=>stop.cancel(),
            // Observe physical EOF even while the face awaits a full stdout.
            // Keep signalling registered groups: a fork already in progress
            // can publish a member after the handler's first killpg snapshot.
            _=shutdown_watch.tick(),if tty||stop.is_cancelled()=>{
                if stop.is_cancelled()||(tty&&stdin_hung_up()) {
                    stop_cli(&stop);
                }
            },
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
    let mut acp = false;
    // Default profile: BBOXY450_HARNESS_PROFILE, then ~/.bboxy450-harness/default-profile, then nemotron.
    let mut profile = std::env::var("BBOXY450_HARNESS_PROFILE")
        .ok()
        .filter(|p| !p.trim().is_empty())
        .or_else(|| {
            let home = std::env::var_os("HOME")?;
            let text = std::fs::read_to_string(
                PathBuf::from(home).join(".bboxy450-harness/default-profile"),
            )
            .ok()?;
            Some(text.trim().to_owned()).filter(|t| !t.is_empty())
        })
        .unwrap_or_else(|| "nemotron".to_owned());
    let mut cwd = std::env::current_dir().map_err(|_| "working directory unavailable")?;
    let mut sessions = None;
    let mut scripts = vec![];
    let mut resume = None;
    let mut yolo = false;
    let mut max_rounds = 50;
    let mut append_system_prompt = String::new();
    let mut mcp_config = None;
    let mut replay = None;
    let mut socket_path: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "ir-scan" => return super::ir_scan::cli(args).await,
            "dream" => return crate::dream::cli(args).await,
            "reproject" => {
                let source = args.next().ok_or("missing session")?;
                let mut out = None;
                while let Some(arg) = args.next() {
                    match arg.as_str() {
                        "--out" => {
                            out = Some(PathBuf::from(
                                args.next().ok_or("missing output directory")?,
                            ))
                        }
                        _ => return Err(format!("unknown reproject option {arg}")),
                    }
                }
                let root = sessions.clone().unwrap_or_else(|| {
                    PathBuf::from(std::env::var_os("HOME").unwrap_or_default())
                        .join(".bboxy450-harness/sessions")
                });
                let file = if source.ends_with(".jsonl") {
                    PathBuf::from(source)
                } else {
                    let id: SessionId =
                        SessionId(source.parse().map_err(|_| "invalid source session")?);
                    root.join(format!("{id}.jsonl"))
                };
                let path = crate::import::reproject::run_cli(file, out.unwrap_or(root))
                    .await
                    .map_err(|e| e.to_string())?;
                println!("{}", path.display());
                return Ok(0);
            }
            "smoke-live" => return Ok(super::smoke::run().await),
            #[cfg(feature = "acp")]
            "smoke-live-acp" => return Ok(super::smoke::run_acp().await),
            "seat" => seat = true,
            "acp" => acp = true,
            "-p" | "--prompt" => prompt = Some(args.next().ok_or("missing prompt")?),
            "--json" => json = true,
            "--profile" => profile = args.next().ok_or("missing profile")?,
            "--cwd" => cwd = PathBuf::from(args.next().ok_or("missing cwd")?),
            "--sessions" => {
                sessions = Some(PathBuf::from(args.next().ok_or("missing sessions path")?))
            }
            "--scripted" => scripts.push(PathBuf::from(args.next().ok_or("missing SSE path")?)),
            "--socket" => socket_path = Some(args.next().ok_or("missing socket path")?.into()),
            "--resume" => resume = Some(args.next().ok_or("missing session")?),
            "--replay" => replay = Some(PathBuf::from(args.next().ok_or("missing replay path")?)),
            "--yolo" => yolo = true,
            "--append-system-prompt" => {
                append_system_prompt = args.next().ok_or("missing system prompt text")?
            }
            "--mcp-config" => {
                mcp_config = Some(PathBuf::from(args.next().ok_or("missing MCP config path")?))
            }
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
                    serde_json::to_string_pretty(&crate::protocol::schema_value()).unwrap()
                );
                return Ok(0);
            }
            "usage" => {
                let mut paths = vec![];
                for arg in args.by_ref() {
                    if arg == "--json" {
                        json = true;
                    } else {
                        paths.push(PathBuf::from(arg));
                    }
                }
                if paths.is_empty() {
                    return Err("usage requires session directories or JSONL files".into());
                }
                let report =
                    crate::runtime::blocking(move || super::usage::report_paths(&paths)).await?;
                if json {
                    println!(
                        "{}",
                        serde_json::to_string_pretty(&report).map_err(|e| e.to_string())?
                    );
                } else {
                    print!("{}", super::usage::render(&report));
                }
                return Ok(0);
            }
            "import" => {
                // bboxy450-harness import [claude|codex] FILE: project a native session into our log
                let first = args.next().ok_or("missing native log path")?;
                let (native, path) = match crate::import::Native::parse(&first) {
                    Some(native) => (
                        Some(native),
                        PathBuf::from(args.next().ok_or("missing native log path")?),
                    ),
                    None => (None, PathBuf::from(first)),
                };
                let root = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
                    .join(".bboxy450-harness");
                let sessions = sessions.clone().unwrap_or_else(|| root.join("sessions"));
                let session = crate::import::import_file(native, &path, sessions.clone()).await?;
                println!("{}", sessions.join(format!("{session}.jsonl")).display());
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
                    "bboxy450-harness -p PROMPT --json [--profile NAME] [--cwd DIR]\nbboxy450-harness --json --resume CLOSED_SESSION\nbboxy450-harness seat [--profile NAME] [--socket PATH] [--resume SESSION] [--cwd DIR]   (in a seat: /profiles, /profile NAME) [--append-system-prompt TEXT] [--mcp-config PATH]\nbboxy450-harness seat --replay LOG.jsonl   (render a stored log through the seat face)\n--scripted SSE (repeat for rounds) --sessions DIR --yolo\nbboxy450-harness usage [--json] PATH…"
                );
                return Ok(0);
            }
            _ => return Err(format!("unknown option {arg}")),
        }
    }
    if let Some(path) = replay {
        // seat --replay LOG: render a stored log through the seat face and exit
        if !seat {
            return Err("--replay needs seat".into());
        }
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|_| "replay log unavailable")?;
        let mut projection = super::seat::LineProjection::new(super::seat::Style::detect(false));
        let mut bytes = vec![];
        for line in text.lines().filter(|l| !l.trim().is_empty()) {
            let envelope: Envelope =
                serde_json::from_str(line).map_err(|_| "invalid replay log JSON")?;
            bytes.extend(projection.render(&envelope));
        }
        bytes.push(b'\n');
        let mut stdout = tokio::io::stdout();
        stdout
            .write_all(&bytes)
            .await
            .map_err(|_| "stdout closed")?;
        stdout.flush().await.map_err(|_| "stdout closed")?;
        return Ok(0);
    }
    if !acp && !seat && prompt.is_none() && !(json && resume.is_some()) {
        return Err("provide -p PROMPT --json, or seat".into());
    }
    #[cfg(not(feature = "acp"))]
    if acp {
        return Err("ACP requires the acp feature".into());
    }
    let root = PathBuf::from(std::env::var_os("HOME").ok_or("HOME unavailable")?)
        .join(".bboxy450-harness");
    let sessions = sessions.unwrap_or_else(|| root.join("sessions"));
    let redactor = Set::default();
    crate::capability::register(&redactor).await;
    if !acp && !seat && prompt.is_none() {
        if let Some(notice) = redactor.capability_notice() {
            eprintln!("Notice: {notice}");
        }
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
            let bytes = event
                .wire_bytes()
                .map_err(|_| "event serialization failed")?;
            stdout
                .write_all(&bytes)
                .await
                .map_err(|_| "output unavailable")?;
        }
        stdout.flush().await.map_err(|_| "output unavailable")?;
        return Ok(0);
    }
    let defer_acp_profile = acp && scripts.is_empty();
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
            Ok(p) if defer_acp_profile => {
                // Advertise capabilities without executing a credential command.
                // Host resolves the real provider inside its cancellable factory.
                let client = p
                    .build_provider(
                        crate::redact::Secret::new(String::new(), &redactor),
                        redactor.clone(),
                        Default::default(),
                    )
                    .map_err(|e| e.message)?;
                (client, p.model.clone(), p.policy())
            }
            Ok(p) => match p.key(&redactor, stop.child_token()).await {
                Ok(key) => {
                    let options = provider::Options {
                        retry_deadline: p.retry_deadline.map(std::time::Duration::from_secs),
                        ..Default::default()
                    };
                    let client = p
                        .build_provider(key, redactor.clone(), options)
                        .map_err(|e| e.message)?;
                    (client, p.model.clone(), p.policy())
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
            vcs: None,
            dream: None,
            source: None,
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
        profile_error: if defer_acp_profile {
            Some("profile resolution pending".into())
        } else {
            (model == "unconfigured").then_some("profile could not be resolved".into())
        },
        model,
        policy,
        redactor,
        max_rounds,
    };
    let mut launch = tokio::select! {biased;_=stop.cancelled()=>return Ok(0),launch=read_launch(append_system_prompt,mcp_config)=>launch};
    launch.yolo = yolo;
    let host = Host::new(config.clone()).with_launch(launch);
    if acp {
        #[cfg(feature = "acp")]
        {
            super::acp::serve_with_host(
                tokio::io::stdin(),
                tokio::io::stdout(),
                config,
                host,
                stop.child_token(),
            )
            .await
            .map_err(|_| "ACP stdio failed")?;
            return Ok(0);
        }
        #[cfg(not(feature = "acp"))]
        {
            return Err("ACP requires the acp feature".into());
        }
    }
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
    let mut gateway = if resume.is_some() {
        host.open(session)
            .await
            .map_err(|_| "session could not be opened")?
    } else {
        host.create(start)
            .await
            .map_err(|_| "session could not be created")?
            .0
    };
    let (switch_tx, mut switches) = tokio::sync::mpsc::channel(64);
    if seat {
        gateway = gateway.with_profile_switch(switch_tx);
    }
    let mut socket = super::seat::bind_session(&root, gateway.clone(), resume.is_some()).await?;
    let mut alias = match &socket_path {
        Some(path) => Some(super::seat::bind_alias(path, gateway.clone()).await?),
        None => None,
    };
    let mut session = session;
    let mut subscription = gateway
        .subscribe(Subscribe {
            session,
            after: None,
            ephemeral: seat && !json,
        })
        .map_err(|_| "subscribe refused")?;
    let profiles_root = root.join("profiles");
    let stdin_was_tty = unsafe { libc::isatty(libc::STDIN_FILENO) } == 1;
    let mut line_projection =
        super::seat::LineProjection::new(super::seat::Style::detect(seat && stdin_was_tty));
    let mut stdout = tokio::io::stdout();
    let report_state =
        seat && !json && stdin_was_tty && unsafe { libc::isatty(libc::STDOUT_FILENO) } == 1;
    let _terminal_input = report_state
        .then(super::seat::TerminalInput::enable)
        .transpose()
        .map_err(|_| "terminal input unavailable")?;
    let mut state = super::seat::StateReporter::new(config.redactor.clone());
    if report_state {
        stdout
            .write_all(&state.report())
            .await
            .map_err(|_| "output unavailable")?;
        stdout.flush().await.map_err(|_| "output unavailable")?;
    }
    let mut input = tokio::io::stdin();
    let mut input_bytes = [0; 4096];
    let mut input_filter = super::seat::InputFilter::default();
    ready.store(true, std::sync::atomic::Ordering::Release);
    let mut stopping = false;
    let mut running = false;
    let mut failed = false;
    let op = |session, op| OpEnvelope {
        v: PROTOCOL,
        session,
        op_id: OpId::new(),
        op,
    };
    if let Some(prompt) = prompt {
        gateway
            .send(op(
                session,
                Op::UserTurn {
                    blocks: vec![InputBlock::Text { text: prompt }],
                    mode: TurnMode::Queue,
                },
            ))
            .map_err(|_| "turn refused")?;
    }
    'events: loop {
        let heartbeat = state.heartbeat_deadline();
        tokio::select! {biased;
         _=stop.cancelled(),if !stopping=>{let _=gateway.send(op(session,Op::Close));stopping=true;},
         _=tokio::signal::ctrl_c()=>{input_filter.interrupt();if running{let _=gateway.send(op(session,Op::Interrupt{turn:None}));}},
         _=tokio::time::sleep_until(heartbeat.unwrap_or_else(tokio::time::Instant::now)),if report_state&&heartbeat.is_some()=>{
          if let Some(bytes)=state.heartbeat()
              && (stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err())
          {stop_cli(&stop);let _=gateway.send(op(session,Op::Close));break 'events;}
         },
         Some(request)=switches.recv(),if !stopping=>{
            // Admission was fenced against the gateway that received the op.
            // Never retarget a queued old-session request after a fork.
            if request.env.session != session { continue; }
            let Op::SetProfile { profile: name } = request.env.op else { continue; };
            match host.switch(&gateway, name.clone()).await {
                Ok(next) => {
                    socket.retire().await.map_err(|_| "socket retirement failed")?;
                    if let Some(old) = alias.take() { old.retire().await.map_err(|_| "seat socket retirement failed")?; }
                    let hello = next.hello();
                    session = hello.meta.session;
                    gateway = next.with_profile_switch(gateway.profile_switch_sender().unwrap());
                    socket = super::seat::bind_session(&root, gateway.clone(), false).await?;
                    if let Some(path) = &socket_path { alias = Some(super::seat::bind_alias(path, gateway.clone()).await?); }
                    subscription = gateway.subscribe(Subscribe { session, after: Some((hello.epoch, hello.head)), ephemeral: seat && !json }).map_err(|_| "subscribe refused")?;
                    running = false;
                    if report_state {
                        state = super::seat::StateReporter::new(config.redactor.clone());
                        for event in &gateway.hub.projection().log { state.apply(&event.event); }
                        stdout.write_all(&state.report()).await.map_err(|_| "output unavailable")?;
                    }
                    stdout.write_all(format!("switched to profile {name} · session {session}\n").as_bytes()).await.map_err(|_| "output unavailable")?;
                    stdout.flush().await.map_err(|_| "output unavailable")?;
                }
                Err(error) => {
                    stdout.write_all(format!("could not switch to {name}: {error:?}\n").as_bytes()).await.map_err(|_| "output unavailable")?;
                    stdout.flush().await.map_err(|_| "output unavailable")?;
                }
            }
         },
         read=input.read(&mut input_bytes),if seat&&!stopping=>{
           if read.is_err(){stop_cli(&stop);}
           let actions = match read {
               Ok(n) if n > 0 => input_filter.feed(&input_bytes[..n], report_state),
               _ => input_filter.finish(),
           };
           for action in actions {
            let text = match action {
                super::seat::InputAction::Line(text) => text,
                super::seat::InputAction::Probe => {
                    if report_state && (stdout.write_all(&state.report()).await.is_err()||stdout.flush().await.is_err()) {
                        stop_cli(&stop);let _=gateway.send(op(session,Op::Close));break 'events;
                    }
                    continue;
                }
                super::seat::InputAction::Echo(bytes) => {
                    if stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err() {
                        stop_cli(&stop);let _=gateway.send(op(session,Op::Close));break 'events;
                    }
                    continue;
                }
                super::seat::InputAction::Eof => {
                    if stdin_was_tty && stdin_hung_up(){stop_cli(&stop);}
                    let _=gateway.send(op(session,Op::Close));stopping=true;
                    break;
                }
            };
            if let Some(command)=text.trim().strip_prefix('/'){
                let reply = if let Some(name) = switch_target(command) {
                    // any backend to any backend: fork at head onto NAME, follow the new session
                    if running {
                        "finish or /interrupt the turn before switching\n".to_owned()
                    } else {
                        match gateway.send(op(session, Op::SetProfile { profile: name.to_owned() })) {
                            Ok(_) => continue,
                            Err(error) => format!("could not switch to {name}: {error:?}\n"),
                        }
                    }
                } else {
                    let (request,mut reply)=seat_command(command,&profiles_root);
                    if let Some(request)=request&&gateway.send(op(session,request)).is_err(){reply="refused\n".into();}
                    reply
                };
                if stdout.write_all(reply.as_bytes()).await.is_err()||stdout.flush().await.is_err(){stop_cli(&stop);let _=gateway.send(op(session,Op::Close));break 'events;}
            }else if !text.is_empty(){let _=gateway.send(op(session,Op::UserTurn{blocks:vec![InputBlock::Text{text}],mode:TurnMode::Queue}));}}
         },
         event=subscription.recv()=>{let Some(event)=event else{break;};match &event.event{Event::TurnStarted{..}=>running=true,Event::TurnEnded{reason,..}=>{running=false;failed|=matches!(reason,StopReason::Error{..});if !seat&&!stopping{let _=gateway.send(op(session,Op::Close));stopping=true;}},_=>{}}
          let bytes=if json {event.wire_bytes().map_err(|_|"event serialization failed")?}else{line_projection.render(&event)};
          if stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err(){stop_cli(&stop);let _=gateway.send(op(session,Op::Close));break 'events;}
          if report_state && let Some(bytes) = state.apply(&event.event)
              && (stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err())
          {stop_cli(&stop);let _=gateway.send(op(session,Op::Close));break 'events;}
          if matches!(event.event,Event::SessionClosed){break;}
         }
        }
    }
    if let Some(alias) = alias {
        alias
            .close()
            .await
            .map_err(|_| "seat socket shutdown failed")?;
    }
    socket.close().await.map_err(|_| "socket shutdown failed")?;
    host.join().await.map_err(|_| "session actor failed")?;
    // Seat Ctrl-D/Close succeeds even when a prior (including recovered) turn
    // failed. Headless one-shot mode still reports its turn's failure.
    Ok(if failed && !seat { 1 } else { 0 })
}
async fn read_launch(append_system_prompt: String, path: Option<PathBuf>) -> LaunchOptions {
    let mut launch = LaunchOptions {
        append_system_prompt,
        ..Default::default()
    };
    if let Some(path) = path {
        let result = crate::runtime::blocking(move || {
            #[derive(serde::Deserialize)]
            struct File {
                #[serde(rename = "mcpServers")]
                servers: crate::mcp::Servers,
            }
            let bytes = std::fs::read(path)?;
            serde_json::from_slice::<File>(&bytes)
                .map(|file| file.servers)
                .map_err(std::io::Error::other)
        })
        .await;
        match result {
            Ok(servers) => launch.mcp_servers = servers,
            Err(_) => launch
                .notices
                .push("launcher MCP configuration unavailable or invalid".into()),
        }
    }
    launch
}

// Cooked Ctrl-D returns EOF with a healthy terminal. A revoked/hung-up PTY
// returns EOF/EIO with HUP or failed termios; it takes the synchronous path.
fn stdin_hung_up() -> bool {
    tty_hung_up(libc::STDIN_FILENO)
}
fn tty_hung_up(terminal: libc::c_int) -> bool {
    let mut fd = libc::pollfd {
        fd: terminal,
        // Darwin omits POLLHUP when no events are requested.
        events: libc::POLLIN,
        revents: 0,
    };
    let mut termios = std::mem::MaybeUninit::<libc::termios>::uninit();
    unsafe {
        libc::poll(&mut fd, 1, 0);
    }
    fd.revents & (libc::POLLHUP | libc::POLLERR) != 0
        || unsafe { libc::tcgetattr(terminal, termios.as_mut_ptr()) } != 0
}

/// Seat slash commands: `/profile NAME` switches the live session's profile, `/profiles` lists them.
/// `/profile NAME`: the profile a seat switches to (handled by Host::switch).
fn switch_target(command: &str) -> Option<&str> {
    let mut words = command.split_whitespace();
    match (words.next(), words.next(), words.next()) {
        (Some("profile"), Some(name), None) => Some(name),
        _ => None,
    }
}
fn seat_command(command: &str, root: &std::path::Path) -> (Option<Op>, String) {
    let mut words = command.split_whitespace();
    match (words.next(), words.next()) {
        (Some("profile"), Some(name)) => (None, format!("switching to profile {name}\n")),
        (Some("profiles" | "profile"), None) => {
            let mut names: Vec<String> = std::fs::read_dir(root)
                .map(|d| {
                    d.filter_map(|e| {
                        let path = e.ok()?.path();
                        (path.extension()? == "md")
                            .then(|| path.file_stem().map(|s| s.to_string_lossy().into_owned()))
                            .flatten()
                    })
                    .collect()
                })
                .unwrap_or_default();
            names.sort();
            let reply = if names.is_empty() {
                format!("no profiles in {}\n", root.display())
            } else {
                format!("profiles: {} · /profile NAME switches\n", names.join(", "))
            };
            (None, reply)
        }
        _ => (None, "commands: /profiles · /profile NAME · /help\n".into()),
    }
}

#[cfg(test)]
mod tty_tests {
    use super::*;
    #[test]
    fn physical_eof_is_distinct_from_cooked_control_d() {
        use portable_pty::{PtySize, native_pty_system};
        use std::{
            io::Read,
            os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
        };
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: 24,
                cols: 80,
                pixel_width: 0,
                pixel_height: 0,
            })
            .unwrap();
        let mut slave = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NONBLOCK)
            .open(pty.master.tty_name().unwrap())
            .unwrap();
        let fd = slave.as_raw_fd();
        assert!(!tty_hung_up(fd));
        assert_eq!(
            unsafe { libc::write(pty.master.as_raw_fd().unwrap(), [4u8].as_ptr().cast(), 1) },
            1
        );
        let mut byte = [0u8; 1];
        assert_eq!(slave.read(&mut byte).unwrap(), 0);
        assert!(
            !tty_hung_up(fd),
            "cooked Ctrl-D was classified as a physical hangup"
        );
        drop(pty.master);
        // Another concurrently spawning fixture may still be closing an
        // inherited descriptor. Classify only after physical EOF is observed.
        let mut poll = libc::pollfd {
            fd,
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe {
            libc::poll(&mut poll, 1, 2000);
        }
        assert_eq!(
            slave.read(&mut byte).unwrap(),
            0,
            "physical EOF was not observed"
        );
        assert!(
            tty_hung_up(fd),
            "physical PTY EOF was classified as cooked Ctrl-D"
        );
    }
}
