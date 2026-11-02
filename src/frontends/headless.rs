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
use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio_util::sync::CancellationToken;
pub async fn cli() -> i32 {
    let result=async {
        crate::process::install_shutdown_handlers().map_err(|_|"signal handler failed".to_string())?;
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
    while let Some(arg) = args.next() {
        match arg.as_str() {
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
            "--resume" => resume = Some(args.next().ok_or("missing session")?),
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
                    "bboxy450-harness -p PROMPT --json [--profile NAME] [--cwd DIR]\nbboxy450-harness --json --resume CLOSED_SESSION\nbboxy450-harness seat [--profile NAME] [--resume SESSION] [--cwd DIR]   (in a seat: /profiles, /profile NAME) [--append-system-prompt TEXT] [--mcp-config PATH]\n--scripted SSE (repeat for rounds) --sessions DIR --yolo"
                );
                return Ok(0);
            }
            _ => return Err(format!("unknown option {arg}")),
        }
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
            let event = redactor
                .sanitize(&event)
                .map_err(|_| "event sanitization failed")?;
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
    let launch = tokio::select! {biased;_=stop.cancelled()=>return Ok(0),launch=read_launch(append_system_prompt,mcp_config)=>launch};
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
    let socket_path = root.join("run").join(format!("{session}.sock"));
    if resume.is_some() {
        // Successful open owns the session's exclusive writer lease. Its old
        // process is gone, but SIGKILL cannot unlink the bound socket pathname.
        use std::os::unix::fs::{FileTypeExt, MetadataExt};
        match tokio::fs::symlink_metadata(&socket_path).await {
            Ok(meta)
                if meta.file_type().is_socket() && meta.uid() == unsafe { libc::geteuid() } =>
            {
                tokio::fs::remove_file(&socket_path)
                    .await
                    .map_err(|_| "stale session socket unavailable")?;
            }
            Ok(_) => return Err("session socket path is not an owned socket".into()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => return Err("session socket unavailable".into()),
        }
    }
    let socket = crate::gateway::unix::UnixServer::bind(socket_path, gateway.clone())
        .await
        .map_err(|_| "session socket unavailable")?;
    let mut subscription = gateway
        .subscribe(Subscribe {
            session,
            after: None,
            ephemeral: seat && !json,
        })
        .map_err(|_| "subscribe refused")?;
    let profiles_root = root.join("profiles");
    let mut line_projection = super::seat::LineProjection::default();
    let mut stdout = tokio::io::stdout();
    let stdin_was_tty = unsafe { libc::isatty(libc::STDIN_FILENO) } == 1;
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
         line=input.next_line(),if seat&&!stopping=>{match line{Ok(Some(text))=>{
            if let Some(command)=text.trim().strip_prefix('/'){
                let (request,mut reply)=seat_command(command,&profiles_root);
                if let Some(request)=request&&gateway.send(op(request)).is_err(){reply="refused\n".into();}
                if stdout.write_all(reply.as_bytes()).await.is_err(){let _=gateway.send(op(Op::Close));break;}let _=stdout.flush().await;
            }else if !text.is_empty(){let _=gateway.send(op(Op::UserTurn{blocks:vec![InputBlock::Text{text}],mode:TurnMode::Queue}));}},_=>{if stdin_was_tty && stdin_hung_up(){crate::process::kill_owned_groups();}let _=gateway.send(op(Op::Close));stopping=true;}}},
         event=subscription.recv()=>{let Some(event)=event else{break;};match &event.event{Event::TurnStarted{..}=>running=true,Event::TurnEnded{reason,..}=>{running=false;failed|=matches!(reason,StopReason::Error{..});if !seat&&!stopping{let _=gateway.send(op(Op::Close));stopping=true;}},_=>{}}
          let bytes=if json {let mut b=serde_json::to_vec(&event).map_err(|_|"event serialization failed")?;b.push(b'\n');b}else{line_projection.render(&event)};
          if stdout.write_all(&bytes).await.is_err(){let _=gateway.send(op(Op::Close));break;}let _=stdout.flush().await;
          if matches!(event.event,Event::SessionClosed){break;}
         }
        }
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
fn seat_command(command: &str, root: &std::path::Path) -> (Option<Op>, String) {
    let mut words = command.split_whitespace();
    match (words.next(), words.next()) {
        (Some("profile"), Some(name)) => (
            Some(Op::SetProfile {
                profile: name.into(),
            }),
            format!("switching to profile {name}\n"),
        ),
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
