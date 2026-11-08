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
    let mut early = std::env::args().skip(1);
    if early.next().as_deref() == Some("dream") && early.next().as_deref() == Some("inspect-c1") {
        return crate::dream::c1::cli(early).await;
    }
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
    #[cfg(feature = "test-fixtures")]
    let fixture_requested = std::env::args().any(|arg| arg == "--dream-c1-fixture-next-session");
    #[cfg(not(feature = "test-fixtures"))]
    let fixture_requested = false;
    #[cfg(feature = "test-fixtures")]
    let mut fixture_next_session = None;
    #[cfg(feature = "test-fixtures")]
    let mut explicit_cwd = false;
    let mut args = std::env::args().skip(1);
    let mut prompt = None;
    let mut json = false;
    let mut seat = false;
    let mut acp = false;
    let mut explicit_profile = None;
    let mut cwd = std::env::current_dir().map_err(|_| "working directory unavailable")?;
    let mut sessions = None;
    let mut scripts = vec![];
    let mut resume = None;
    let mut yolo = false;
    let mut dream_owner = crate::dream::c1::DreamOwner::Legacy;
    let mut max_rounds = 50;
    let mut append_system_prompt = String::new();
    let mut mcp_config = None;
    let mut replay = None;
    let mut socket_path: Option<PathBuf> = None;
    while let Some(arg) = args.next() {
        // The private fixture cannot enter an unrelated early-return command or
        // any unapproved discovery, credential, resume or ACP setup path.
        if fixture_requested
            && !matches!(
                arg.as_str(),
                "seat"
                    | "--json"
                    | "--cwd"
                    | "--sessions"
                    | "--scripted"
                    | "--socket"
                    | "--dream-owner"
                    | "--dream-c1-fixture-next-session"
                    | "--profile"
                    | "--mcp-config"
                    | "--append-system-prompt"
                    | "--yolo"
            )
        {
            return Err("private Dream C1 fixture options refused".into());
        }
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
            "--profile" => explicit_profile = Some(args.next().ok_or("missing profile")?),
            "--cwd" => {
                cwd = PathBuf::from(args.next().ok_or("missing cwd")?);
                #[cfg(feature = "test-fixtures")]
                {
                    explicit_cwd = true;
                }
            }
            "--sessions" => {
                sessions = Some(PathBuf::from(args.next().ok_or("missing sessions path")?))
            }
            "--scripted" => scripts.push(PathBuf::from(args.next().ok_or("missing SSE path")?)),
            "--socket" => socket_path = Some(args.next().ok_or("missing socket path")?.into()),
            "--resume" => resume = Some(args.next().ok_or("missing session")?),
            "--replay" => replay = Some(PathBuf::from(args.next().ok_or("missing replay path")?)),
            "--yolo" => yolo = true,
            "--dream-owner" => {
                dream_owner =
                    crate::dream::c1::DreamOwner::parse(&args.next().ok_or("missing dream owner")?)
                        .ok_or("invalid dream owner")?;
            }
            #[cfg(feature = "test-fixtures")]
            "--dream-c1-fixture-next-session" => {
                if fixture_next_session.is_some() {
                    return Err("duplicate private Dream C1 fixture option".into());
                }
                let raw = args.next().ok_or("missing private fixture session")?;
                let id = raw
                    .parse::<ulid::Ulid>()
                    .map_err(|_| "invalid private fixture session")?;
                if id.to_string() != raw {
                    return Err("invalid private fixture session".into());
                }
                fixture_next_session = Some(SessionId(id));
            }
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
    #[cfg(feature = "test-fixtures")]
    let (fixture_profile, fixture_input) = if fixture_requested {
        if !seat
            || !json
            || acp
            || scripts.is_empty()
            || dream_owner != crate::dream::c1::DreamOwner::TermC1
            || !explicit_cwd
            || sessions.is_none()
            || socket_path.is_none()
            || resume.is_some()
            || replay.is_some()
        {
            return Err("private Dream C1 fixture requires explicit isolated scripted seat".into());
        }
        let input = DreamC1FixtureInput {
            cwd: cwd.clone(),
            sessions: sessions.as_ref().unwrap().clone(),
            socket: socket_path.as_ref().unwrap().clone(),
            scripts: scripts.clone(),
            profile_name: explicit_profile.clone(),
            mcp_config: mcp_config.clone(),
            next_session: fixture_next_session.unwrap(),
            append_len: append_system_prompt.len(),
        };
        (validate_dream_c1_fixture(input.clone()).await?, Some(input))
    } else {
        (None, None)
    };
    #[cfg(not(feature = "test-fixtures"))]
    let fixture_profile: Option<Profile> = None;
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
    if fixture_requested {
        redactor.no_capability();
    } else {
        crate::capability::register(&redactor).await;
    }
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
    let profile = if fixture_requested {
        fixture_profile
            .as_ref()
            .map_or_else(|| "scripted".to_owned(), |p| p.name.clone())
    } else {
        tokio::select! {biased;_=stop.cancelled()=>return Ok(0),name=crate::onboarding::bootstrap(root.join("profiles"), explicit_profile, scripts.is_empty() && resume.is_none())=>name}
    };
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
        let model = fixture_profile
            .as_ref()
            .map_or_else(|| "scripted".to_owned(), |p| p.model.clone());
        let scripted = provider::ScriptedProvider::new(rounds);
        #[cfg(feature = "test-fixtures")]
        let scripted: Arc<dyn Provider> = if fixture_requested {
            Arc::new(DreamC1FixtureProvider {
                scripted,
                model: model.clone(),
                append: append_system_prompt.clone(),
            })
        } else {
            Arc::new(scripted)
        };
        #[cfg(not(feature = "test-fixtures"))]
        let scripted: Arc<dyn Provider> = Arc::new(scripted);
        (scripted, model, Policy::new(Unattended::Deny))
    } else {
        let p = Profile::path(&profile).map_err(|e| e.message)?;
        let loaded =
            tokio::select! {biased;_=stop.cancelled()=>return Ok(0),p=Profile::load(&p)=>p};
        // Fail before creating a session when local setup cannot launch it.
        // Resolve only the executable, never any credential command.
        #[cfg(feature = "acp-agent")]
        if let Ok(p) = &loaded
            && resume.is_none()
            && p.is_acp_agent()
        {
            let command = &p.agent.as_ref().unwrap().command[0];
            // Match the real ACP spawn path, including login-shell PATH. Direct
            // paths retain Command's existing cwd-relative launch semantics.
            if !command.contains('/') {
                let path = crate::backend::acp_agent::login_path().await;
                crate::backend::acp_agent::find_agent(command, path, &cwd)
                    .await
                    .map_err(|e| format!("{e}; {}", crate::onboarding::setup_guidance(&profile)))?;
            }
        }
        match loaded {
            Err(e) if resume.is_none() => {
                return Err(format!(
                    "{}; {}",
                    e.message,
                    crate::onboarding::setup_guidance(&profile)
                ));
            }
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
    if fixture_requested {
        let path = crate::store::log_path(&sessions, session);
        crate::runtime::blocking(move || {
            if !matches!(std::fs::symlink_metadata(path), Err(e) if e.kind() == std::io::ErrorKind::NotFound) {
                return Err("private Dream C1 fixture refused [initial_log_exists]".to_owned());
            }
            Ok(())
        }).await?;
    }
    let config = Config {
        meta: SessionMeta {
            vcs: None,
            dream: None,
            source: None,
            session,
            cwd,
            // Only an explicitly validated private fixture profile is selected.
            // Host reuses the supplied scripted provider and ordinary metadata.
            profile: if fixture_requested && fixture_profile.is_none() {
                None
            } else {
                Some(profile)
            },
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
    if fixture_requested && !launch.notices.is_empty() {
        return Err("private Dream C1 fixture refused [mcp_load]".into());
    }
    launch.yolo = yolo;
    launch.dream_owner = dream_owner;
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
    // Set when a seat lost its terminal: the session stays open for a resume.
    let mut detached = false;
    'events: loop {
        let heartbeat = state.heartbeat_deadline();
        tokio::select! {biased;
         // A seat that loses its terminal (signal, hangup, supervisor restart) leaves
         // its session open so the next launch resumes it; only a deliberate end closes.
         _=stop.cancelled(),if !stopping=>{if seat{detached=true;break 'events;}let _=gateway.send(op(session,Op::Close));stopping=true;},
         _=tokio::signal::ctrl_c()=>{input_filter.interrupt();if running{let _=gateway.send(op(session,Op::Interrupt{turn:None}));}},
         _=tokio::time::sleep_until(heartbeat.unwrap_or_else(tokio::time::Instant::now)),if report_state&&heartbeat.is_some()=>{
          if let Some(bytes)=state.heartbeat()
              && (stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err())
          {stop_cli(&stop);if seat{detached=true;}else{let _=gateway.send(op(session,Op::Close));}break 'events;}
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
                        stop_cli(&stop);if seat{detached=true;}else{let _=gateway.send(op(session,Op::Close));}break 'events;
                    }
                    continue;
                }
                super::seat::InputAction::Echo(bytes) => {
                    if stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err() {
                        stop_cli(&stop);if seat{detached=true;}else{let _=gateway.send(op(session,Op::Close));}break 'events;
                    }
                    continue;
                }
                super::seat::InputAction::Eof => {
                    #[cfg(feature = "test-fixtures")]
                    { fixture_next_session = None; }
                    if stdin_was_tty && stdin_hung_up(){stop_cli(&stop);if seat{detached=true;break 'events;}}
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
                if stdout.write_all(reply.as_bytes()).await.is_err()||stdout.flush().await.is_err(){stop_cli(&stop);if seat{detached=true;}else{let _=gateway.send(op(session,Op::Close));}break 'events;}
            }else if !text.is_empty(){let _=gateway.send(op(session,Op::UserTurn{blocks:vec![InputBlock::Text{text}],mode:TurnMode::Queue}));}}
         },
         event=subscription.recv()=>{let Some(event)=event else{break;};match &event.event{Event::TurnStarted{..}=>running=true,Event::TurnEnded{reason,..}=>{running=false;failed|=matches!(reason,StopReason::Error{..});if !seat&&!stopping{let _=gateway.send(op(session,Op::Close));stopping=true;}},_=>{}}
          let bytes=if json {event.wire_bytes().map_err(|_|"event serialization failed")?}else{line_projection.render(&event)};
          if stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err(){stop_cli(&stop);if seat{detached=true;}else{let _=gateway.send(op(session,Op::Close));}break 'events;}
          if report_state && let Some(bytes) = state.apply(&event.event)
              && (stdout.write_all(&bytes).await.is_err()||stdout.flush().await.is_err())
          {stop_cli(&stop);if seat{detached=true;}else{let _=gateway.send(op(session,Op::Close));}break 'events;}
          if matches!(event.event,Event::SessionClosed){
           #[cfg(feature = "test-fixtures")]
           if !stopping && !stop.is_cancelled() && let Some(next_session) = fixture_next_session.take() {
            // This point follows the actual durable close's stdout flush. Keep
            // the same Host/PID/input, retire A, and create exactly one fresh B.
            socket.retire().await.map_err(|_| "fixture socket retirement failed")?;
            if let Some(old) = alias.take() { old.retire().await.map_err(|_| "fixture alias retirement failed")?; }
            let checked = validate_dream_c1_fixture(fixture_input.as_ref().unwrap().clone()).await?;
            if checked.as_ref().map(|p| (&p.name, &p.model)) != fixture_profile.as_ref().map(|p| (&p.name, &p.model)) {
                return Err("private Dream C1 fixture refused [profile_changed]".into());
            }
            let next = host.create(op(next_session, Op::StartSession {
                cwd: config.meta.cwd.clone(), profile: config.meta.profile.clone(), fork: None,
            })).await.map_err(|_| "fixture next session could not be created")?.0;
            let hello = next.hello();
            session = hello.meta.session;
            gateway = match gateway.profile_switch_sender() {
                Some(sender) => next.with_profile_switch(sender), None => next,
            };
            socket = super::seat::bind_session(&root, gateway.clone(), false).await?;
            if let Some(path) = &socket_path { alias = Some(super::seat::bind_alias(path, gateway.clone()).await?); }
            subscription = gateway.subscribe(Subscribe {
                session, after: None, ephemeral: false,
            }).map_err(|_| "fixture next subscription refused")?;
            stopping = false;
            running = false;
            input_filter = super::seat::InputFilter::default();
            continue 'events;
           }
           break;
          }
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
    if detached {
        // No Close was sent, so the actor never finishes: exit as an abrupt stop,
        // which the store already recovers on the next open.
        return Ok(0);
    }
    host.join().await.map_err(|_| "session actor failed")?;
    // Seat Ctrl-D/Close succeeds even when a prior (including recovered) turn
    // failed. Headless one-shot mode still reports its turn's failure.
    Ok(if failed && !seat { 1 } else { 0 })
}
#[cfg(feature = "test-fixtures")]
fn dream_c1_fixture_environment_entry(name: &std::ffi::OsStr, value: &std::ffi::OsStr) -> bool {
    if matches!(
        name.to_str(),
        Some(
            "HOME"
                | "PATH"
                | "LANG"
                | "LC_ALL"
                | "TERM"
                | "TMPDIR"
                | "TZ"
                | "BBOXY_SEAT_ID"
                | "BBOXY_GENERATION"
                | "BBOXY_ROLE"
                | "BBOXY_SOCK"
                | "BBOXY_CAPABILITY_FILE"
                | "TERM_PROGRAM"
                | "TERM_PROGRAM_VERSION"
                | "ZDOTDIR"
                | "CODEX_HOME"
                | "CLAUDE_CONFIG_DIR"
        )
    ) {
        return true;
    }
    // CoreFoundation initializes this locale metadata while dyld loads the
    // linked framework, even when the parent supplies a cleared environment.
    // Admit only the native UID/script/region triple, not arbitrary new names.
    if cfg!(target_os = "macos") && name == "__CF_USER_TEXT_ENCODING" {
        let Some(text) = value.to_str().filter(|v| v.len() <= 64) else {
            return false;
        };
        let mut parts = text.split(':');
        let number = |part: Option<&str>| {
            let part = part?;
            if let Some(hex) = part.strip_prefix("0x") {
                u32::from_str_radix(hex, 16).ok()
            } else if part.bytes().all(|b| b.is_ascii_digit()) {
                part.parse::<u32>().ok()
            } else {
                None
            }
        };
        return number(parts.next()) == Some(unsafe { libc::geteuid() })
            && number(parts.next()).is_some()
            && number(parts.next()).is_some()
            && parts.next().is_none();
    }
    false
}

#[cfg(feature = "test-fixtures")]
struct DreamC1FixtureProvider {
    scripted: provider::ScriptedProvider,
    model: String,
    append: String,
}
#[cfg(feature = "test-fixtures")]
impl Provider for DreamC1FixtureProvider {
    fn id(&self) -> &str {
        self.scripted.id()
    }
    fn caps(&self) -> provider::Caps {
        self.scripted.caps()
    }
    fn stream(
        &self,
        request: provider::ModelRequest,
        cancel: CancellationToken,
    ) -> std::pin::Pin<Box<dyn futures_core::Stream<Item = provider::ModelEvent> + Send>> {
        // Inspect the real production request boundary; never synthesize a
        // successful turn after losing managed model/bootstrap metadata.
        let system = request.messages.first().and_then(|m| m["content"].as_str());
        if request.model != self.model || !system.is_some_and(|s| s.ends_with(&self.append)) {
            return Box::pin(futures_util::stream::once(async {
                provider::ModelEvent::Failed(ProviderError::new(
                    ErrorClass::Config,
                    "private fixture managed request metadata lost",
                ))
            }));
        }
        self.scripted.stream(request, cancel)
    }
}

#[cfg(feature = "test-fixtures")]
fn dream_c1_fixture_mcp(servers: &crate::mcp::Servers) -> Result<(), String> {
    let refused = || "private Dream C1 fixture refused [mcp_environment]".to_owned();
    for server in servers.values() {
        if server.command.is_empty()
            || server.command.len() > 4096
            || server
                .args
                .iter()
                .any(|a| a.len() > 8192 || a.contains('\0'))
        {
            return Err("private Dream C1 fixture refused [mcp_command]".into());
        }
        for (name, source) in &server.env {
            if name.len() > 256
                || source.len() > 8192
                || source.contains('\0')
                || source.starts_with('!')
                || name == "BBOXY_CAPABILITY"
                || name.ends_with("API_KEY")
            {
                return Err(refused());
            }
            if name.starts_with("BBOXY_") {
                if !matches!(
                    name.as_str(),
                    "BBOXY_SEAT_ID"
                        | "BBOXY_GENERATION"
                        | "BBOXY_ROLE"
                        | "BBOXY_SOCK"
                        | "BBOXY_CAPABILITY_FILE"
                ) {
                    return Err(refused());
                }
                let expected = std::env::var(name).map_err(|_| refused())?;
                if source != &expected && source != &format!("${name}") {
                    return Err(refused());
                }
            }
            if let Some(name) = source.strip_prefix('$') {
                let value = std::env::var_os(name).ok_or_else(refused)?;
                if !dream_c1_fixture_environment_entry(std::ffi::OsStr::new(name), &value) {
                    return Err(refused());
                }
            }
        }
    }
    Ok(())
}

#[cfg(feature = "test-fixtures")]
#[derive(Clone)]
struct DreamC1FixtureInput {
    cwd: PathBuf,
    sessions: PathBuf,
    socket: PathBuf,
    scripts: Vec<PathBuf>,
    profile_name: Option<String>,
    mcp_config: Option<PathBuf>,
    next_session: SessionId,
    append_len: usize,
}

#[cfg(feature = "test-fixtures")]
async fn validate_dream_c1_fixture(input: DreamC1FixtureInput) -> Result<Option<Profile>, String> {
    let DreamC1FixtureInput {
        cwd,
        sessions,
        socket,
        scripts,
        profile_name,
        mcp_config,
        next_session,
        append_len,
    } = input;
    fn deny(code: &str) -> String {
        format!("private Dream C1 fixture refused [{code}]")
    }
    for (name, value) in std::env::vars_os() {
        if !dream_c1_fixture_environment_entry(&name, &value) {
            return Err(deny("environment_entry"));
        }
    }
    if append_len > 256 * 1024 {
        return Err(deny("append_size"));
    }
    for name in ["BBOXY_ROLE", "TERM_PROGRAM", "TERM_PROGRAM_VERSION"] {
        if let Some(value) = std::env::var_os(name)
            && !value.to_str().is_some_and(|s| {
                s.len() <= 256
                    && (name != "BBOXY_ROLE" || !s.is_empty())
                    && !s.chars().any(char::is_control)
            })
        {
            return Err(deny("environment_metadata"));
        }
    }
    const MANAGED: [&str; 5] = [
        "BBOXY_SEAT_ID",
        "BBOXY_GENERATION",
        "BBOXY_ROLE",
        "BBOXY_SOCK",
        "BBOXY_CAPABILITY_FILE",
    ];
    let managed = MANAGED.iter().any(|name| std::env::var_os(name).is_some());
    if managed {
        if MANAGED.iter().any(|name| std::env::var_os(name).is_none()) {
            return Err(deny("managed_fields"));
        }
        let seat = std::env::var("BBOXY_SEAT_ID").map_err(|_| deny("managed_seat"))?;
        if !seat
            .parse::<ulid::Ulid>()
            .is_ok_and(|id| id.to_string() == seat)
        {
            return Err(deny("managed_seat"));
        }
        let generation =
            std::env::var("BBOXY_GENERATION").map_err(|_| deny("managed_generation"))?;
        if generation.len() > 20
            || !generation.bytes().all(|b| b.is_ascii_digit())
            || !generation.parse::<u64>().is_ok_and(|n| n != 0)
        {
            return Err(deny("managed_generation"));
        }
    }
    let home = PathBuf::from(std::env::var_os("HOME").ok_or_else(|| deny("home_missing"))?);
    let dirs = ["ZDOTDIR", "CODEX_HOME", "CLAUDE_CONFIG_DIR", "TMPDIR"]
        .into_iter()
        .filter_map(|name| std::env::var_os(name).map(|p| (name, PathBuf::from(p))))
        .collect::<Vec<_>>();
    let managed_socket = std::env::var_os("BBOXY_SOCK").map(PathBuf::from);
    let capability = std::env::var_os("BBOXY_CAPABILITY_FILE").map(PathBuf::from);
    crate::runtime::blocking(move || {
        use std::{fs::{Metadata, OpenOptions}, io::Read, os::unix::fs::{FileTypeExt, MetadataExt, OpenOptionsExt}, path::{Component, Path}};
        struct Private { root: PathBuf }
        impl Private {
            fn lexical(path: &Path, label: &str) -> Result<(), String> {
                if !path.is_absolute() || path.as_os_str().as_encoded_bytes().len() > 4096
                    || path.components().any(|c| !matches!(c, Component::RootDir | Component::Normal(_))) {
                    return Err(deny(&format!("{label}_path")));
                }
                Ok(())
            }
            fn directory(&self, path: &Path, label: &str, optional: bool) -> Result<bool, String> {
                Self::lexical(path, label)?;
                if !path.starts_with(&self.root) { return Err(deny("mixed_root")); }
                let mut walked = PathBuf::from("/");
                for part in path.components() {
                    if let Component::Normal(part) = part { walked.push(part); } else { continue; }
                    let m = match std::fs::symlink_metadata(&walked) {
                        Ok(m) => m,
                        Err(e) if optional && e.kind() == std::io::ErrorKind::NotFound && walked.starts_with(&self.root) => return Ok(false),
                        Err(_) => return Err(deny(&format!("{label}_missing"))),
                    };
                    if m.file_type().is_symlink() { return Err(deny(&format!("{label}_symlink"))); }
                    if !m.is_dir() { return Err(deny(&format!("{label}_kind"))); }
                    if walked.starts_with(&self.root) {
                        if m.uid() != unsafe { libc::geteuid() } { return Err(deny(&format!("{label}_owner"))); }
                        if m.mode() & 0o7777 != 0o700 { return Err(deny(&format!("{label}_mode"))); }
                    }
                }
                Ok(true)
            }
            fn data(&self, path: &Path, label: &str, maximum: u64) -> Result<Metadata, String> {
                Self::lexical(path, label)?;
                self.directory(path.parent().ok_or_else(|| deny("file_parent"))?, label, false)?;
                let m = std::fs::symlink_metadata(path).map_err(|_| deny(&format!("{label}_missing")))?;
                if m.file_type().is_symlink() { return Err(deny(&format!("{label}_symlink"))); }
                if !m.is_file() { return Err(deny(&format!("{label}_kind"))); }
                if m.uid() != unsafe { libc::geteuid() } { return Err(deny(&format!("{label}_owner"))); }
                if m.mode() & 0o7777 != 0o600 { return Err(deny(&format!("{label}_mode"))); }
                if m.nlink() != 1 { return Err(deny(&format!("{label}_links"))); }
                if m.len() == 0 || m.len() > maximum { return Err(deny(&format!("{label}_size"))); }
                Ok(m)
            }
            fn read(&self, path: &Path, label: &str, maximum: u64) -> Result<Vec<u8>, String> {
                let before = self.data(path, label, maximum)?;
                let mut f = OpenOptions::new().read(true).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK | libc::O_CLOEXEC)
                    .open(path).map_err(|_| deny(&format!("{label}_open")))?;
                let same = |a: &Metadata, b: &Metadata| a.dev() == b.dev() && a.ino() == b.ino()
                    && a.len() == b.len() && a.mtime() == b.mtime() && a.mtime_nsec() == b.mtime_nsec()
                    && a.ctime() == b.ctime() && a.ctime_nsec() == b.ctime_nsec();
                if !same(&before, &f.metadata().map_err(|_| deny("file_metadata"))?) { return Err(deny("file_changed")); }
                let mut bytes = Vec::new();
                (&mut f).take(maximum + 1).read_to_end(&mut bytes).map_err(|_| deny(&format!("{label}_read")))?;
                let after = self.data(path, label, maximum)?;
                if bytes.len() as u64 != before.len() || !same(&before, &after)
                    || !same(&before, &f.metadata().map_err(|_| deny("file_metadata"))?) { return Err(deny("file_changed")); }
                Ok(bytes)
            }
            fn absent(&self, path: &Path, label: &str) -> Result<(), String> {
                Self::lexical(path, label)?;
                if !path.starts_with(&self.root) { return Err(deny("mixed_root")); }
                if !matches!(std::fs::symlink_metadata(path), Err(e) if e.kind() == std::io::ErrorKind::NotFound) {
                    return Err(deny(&format!("{label}_exists")));
                }
                Ok(())
            }
        }
        let profile_path = profile_name.as_ref().map(|name| Profile::file_in(&home.join(".bboxy450-harness/profiles"), name)
            .map_err(|_| deny("profile_name"))).transpose()?;
        let mut targets = vec![home.clone(), cwd.clone(), sessions.clone(), socket.clone()];
        targets.extend(scripts.iter().cloned());
        targets.extend(dirs.iter().map(|(_, p)| p.clone()));
        targets.extend(profile_path.iter().cloned());
        targets.extend(mcp_config.iter().cloned());
        targets.extend(capability.iter().cloned());
        targets.extend(managed_socket.iter().cloned());
        let mut root = home.clone();
        for target in &targets {
            Private::lexical(target, "input")?;
            while !target.starts_with(&root) {
                if !root.pop() { return Err(deny("mixed_root")); }
            }
        }
        let private = Private { root };
        private.directory(&private.root, "root", false)?;
        for (label, dir) in [("home", &home), ("cwd", &cwd), ("sessions", &sessions)] {
            private.directory(dir, label, false)?;
        }
        for (_, dir) in &dirs { private.directory(dir, "overlay", false)?; }
        for suffix in [".bboxy450-harness", ".bboxy450-harness/profiles", ".bboxy450-harness/run"] {
            private.directory(&home.join(suffix), "runtime", true)?;
        }
        private.directory(socket.parent().ok_or_else(|| deny("socket_parent"))?, "socket_parent", false)?;
        if socket.as_os_str().as_encoded_bytes().len() > 96 { return Err(deny("socket_size")); }
        private.absent(&socket, "socket")?;
        private.absent(&crate::store::log_path(&sessions, next_session), "next_log")?;
        private.absent(&home.join(format!(".bboxy450-harness/run/{next_session}.sock")), "next_socket")?;
        for script in &scripts { private.data(script, "script", 1024 * 1024)?; }
        if let Some(path) = &managed_socket {
            private.directory(path.parent().ok_or_else(|| deny("managed_socket_parent"))?, "managed_socket", false)?;
            let m = std::fs::symlink_metadata(path).map_err(|_| deny("managed_socket_missing"))?;
            if !m.file_type().is_socket() || m.uid() != unsafe { libc::geteuid() }
                || m.mode() & 0o7777 != 0o600 || m.nlink() != 1 { return Err(deny("managed_socket_metadata")); }
        }
        if let Some(path) = &capability {
            let config = mcp_config.as_ref().ok_or_else(|| deny("capability_overlay"))?;
            if path.parent() != config.parent() { return Err(deny("capability_overlay")); }
            // Metadata ONLY. Capability bytes belong to the ordinary admitted MCP bridge.
            private.data(path, "capability", 4096)?;
        }
        if let Some(path) = &mcp_config {
            #[derive(serde::Deserialize)]
            #[serde(deny_unknown_fields)]
            struct File { #[serde(rename = "mcpServers")] servers: crate::mcp::Servers }
            let bytes = private.read(path, "mcp", 256 * 1024)?;
            let file: File = serde_json::from_slice(&bytes).map_err(|_| deny("mcp_json"))?;
            dream_c1_fixture_mcp(&file.servers)?;
        }
        let profile = if let Some(path) = &profile_path {
            let bytes = private.read(path, "profile", 256 * 1024)?;
            let profile = Profile::parse(std::str::from_utf8(&bytes).map_err(|_| deny("profile_utf8"))?)
                .map_err(|_| deny("profile_parse"))?;
            if Some(&profile.name) != profile_name.as_ref() { return Err(deny("profile_name")); }
            if profile.auto_compact || profile.is_acp_agent() || profile.agent.is_some() { return Err(deny("profile_mode")); }
            if profile.api_key.is_some() || profile.api_key_env.is_some() { return Err(deny("profile_credentials")); }
            dream_c1_fixture_mcp(&profile.mcp_servers)?;
            Some(profile)
        } else { None };
        // Bind_session's ordinary creation defaults are not a private-mode
        // allocation contract. Prepare only these explicit fixture-owned paths.
        for suffix in [".bboxy450-harness", ".bboxy450-harness/run"] {
            let path = home.join(suffix);
            if !private.directory(&path, "runtime", true)? {
                use std::os::unix::fs::DirBuilderExt;
                std::fs::DirBuilder::new().mode(0o700).create(&path).map_err(|_| deny("runtime_create"))?;
                private.directory(&path, "runtime", false)?;
            }
        }
        Ok(profile)
    }).await
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

struct TrialCloseGuard(Option<crate::gateway::InProc>);
impl Drop for TrialCloseGuard {
    fn drop(&mut self) {
        if let Some(gateway) = self.0.take() {
            for op in [Op::Interrupt { turn: None }, Op::Close] {
                let _ = gateway.send(OpEnvelope {
                    v: PROTOCOL,
                    session: gateway.hello().meta.session,
                    op_id: OpId::new(),
                    op,
                });
            }
        }
    }
}
/// Trial continuation uses the ordinary actor/Store/fork/provider path, with the
/// opaque capability supplied only by trusted scratch orchestration. No native
/// handoff, extra instruction authority, live profile lookup or recall is added.
pub(crate) async fn trial_continuation(
    config: Config,
    capability: crate::tools::TrialCapability,
    blocks: Vec<InputBlock>,
    cancel: CancellationToken,
    token_budget: u64,
    deadline: tokio::time::Instant,
) -> std::io::Result<(Vec<Envelope>, bool, bool, Option<tokio::time::Instant>)> {
    let path = crate::store::log_path(&config.sessions, config.meta.session);
    let fork = config.meta.fork.map(|f| f.1).unwrap_or(Seq(0));
    let (gateway, task) = crate::engine::SessionActor::load_with_mcp(
        config,
        OpId::new(),
        None,
        Default::default(),
        LaunchOptions {
            trial: Some(capability.clone()),
            ..Default::default()
        },
    )
    .await?;
    let mut close_guard = TrialCloseGuard(Some(gateway.clone()));
    let hello = gateway.hello();
    let mut sub = gateway
        .subscribe(Subscribe {
            session: hello.meta.session,
            after: Some((hello.epoch, hello.head)),
            ephemeral: false,
        })
        .map_err(|_| std::io::Error::other("trial subscription refused"))?;
    let send = |op| {
        gateway.send(OpEnvelope {
            v: PROTOCOL,
            session: hello.meta.session,
            op_id: OpId::new(),
            op,
        })
    };
    let admitted = !cancel.is_cancelled()
        && tokio::time::Instant::now() < deadline
        && send(Op::UserTurn {
            blocks,
            mode: TurnMode::Queue,
        })
        .is_ok();
    let mut ended_at = None;
    let mut valid = admitted;
    let mut seen = std::collections::BTreeSet::new();
    let mut spend = Some(0u64);
    if admitted {
        loop {
            let event = tokio::select! {biased;
                _=cancel.cancelled()=>{valid=false;None},
                _=tokio::time::sleep_until(deadline)=>{valid=false;None},
                e=sub.recv()=>e,
            };
            let Some(event) = event else {
                valid = false;
                break;
            };
            if event.seq.is_none_or(|s| s <= fork) {
                continue;
            }
            match event.event {
                Event::ItemCompleted {
                    item,
                    body: Item::Usage { usage },
                    ..
                } if usage.scope == Some(UsageScope::Call) && seen.insert(item) => {
                    let n = usage.numbers();
                    spend = spend
                        .zip(n.get("input_tokens").copied())
                        .and_then(|(a, b)| a.checked_add(b))
                        .zip(n.get("output_tokens").copied())
                        .and_then(|(a, b)| a.checked_add(b));
                    if spend.is_none_or(|n| n > token_budget) {
                        valid = false;
                        break;
                    }
                }
                Event::TurnEnded { reason, .. } => {
                    ended_at = Some(tokio::time::Instant::now());
                    valid &= matches!(reason, StopReason::EndTurn);
                    break;
                }
                Event::OpRejected { .. } | Event::SessionClosed => {
                    valid = false;
                    break;
                }
                _ => {}
            }
        }
    }
    if !valid {
        let _ = send(Op::Interrupt { turn: None });
    }
    let _ = send(Op::Close);
    // Never abort this ownership/closure join at a caller deadline. The trial's
    // local retirement owner retains scratch and claim if its caller abandons it.
    let joined = task.await.map_err(std::io::Error::other).and_then(|r| r);
    capability.finish().await;
    close_guard.0 = None;
    valid &= joined.is_ok();
    let events = crate::runtime::blocking(move || {
        use std::io::Read;
        use std::os::unix::fs::OpenOptionsExt;
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_NONBLOCK)
            .open(path)?;
        let mut text = String::new();
        file.read_to_string(&mut text)?;
        text.split_inclusive('\n')
            .map(|line| Envelope::from_line(line).map_err(std::io::Error::other))
            .collect::<std::io::Result<Vec<_>>>()
    })
    .await?;
    let settled = joined.is_ok()
        && events
            .last()
            .is_some_and(|e| matches!(e.event, Event::SessionClosed));
    Ok((events, valid, settled, ended_at))
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

#[cfg(all(test, feature = "test-fixtures"))]
mod dream_c1_fixture_environment_tests {
    use super::dream_c1_fixture_environment_entry;
    use std::ffi::OsStr;
    #[test]
    fn native_locale_entry_validation_rejects_malformed_values_without_dyld_rewriting() {
        let name = OsStr::new("__CF_USER_TEXT_ENCODING");
        for invalid in ["not-native-locale-metadata", "0x0:0x0", "0x0:0x0:0x0:0x0"] {
            assert!(!dream_c1_fixture_environment_entry(
                name,
                OsStr::new(invalid)
            ));
        }
        #[cfg(target_os = "macos")]
        {
            let uid = unsafe { libc::geteuid() };
            assert!(dream_c1_fixture_environment_entry(
                name,
                OsStr::new(&format!("0x{uid:X}:0x0:0x0"))
            ));
            assert!(!dream_c1_fixture_environment_entry(
                name,
                OsStr::new(&format!("0x{:X}:0x0:0x0", uid.wrapping_add(1)))
            ));
        }
        assert!(!dream_c1_fixture_environment_entry(
            OsStr::new("C1_UNAPPROVED_ENV"),
            OsStr::new("unsupported")
        ));
    }
}
