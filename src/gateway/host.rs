// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Async loading boundary. No disk read or recovery occurs in Gateway::subscribe.
use super::*;
use crate::{
    engine::{Config, LaunchOptions, SessionActor},
    protocol::ErrorClass,
    provider::{FailedProvider, ProviderError, profile::Profile},
};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
struct Loaded {
    gateway: InProc,
    start: OpId,
    task: Option<tokio::task::JoinHandle<std::io::Result<()>>>,
}
struct State {
    loaded: Option<Loaded>,
    failed_open: bool,
}
pub struct Host {
    config: Config,
    sessions: std::sync::Mutex<std::collections::BTreeMap<SessionId, Arc<Mutex<State>>>>,
    cancel: CancellationToken,
    launch: LaunchOptions,
}
impl Host {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            sessions: std::sync::Mutex::new(Default::default()),
            cancel: CancellationToken::new(),
            launch: LaunchOptions::default(),
        }
    }
    pub fn with_launch(mut self, launch: LaunchOptions) -> Self {
        self.launch = launch;
        self
    }
    fn entry(&self, session: SessionId) -> Arc<Mutex<State>> {
        self.sessions
            .lock()
            .unwrap()
            .entry(session)
            .or_insert_with(|| {
                Arc::new(Mutex::new(State {
                    loaded: None,
                    failed_open: false,
                }))
            })
            .clone()
    }
    pub async fn create(&self, op: OpEnvelope) -> Result<(InProc, Ack), Refused> {
        self.create_with_mcp(op, Default::default()).await
    }
    pub async fn create_with_mcp(
        &self,
        op: OpEnvelope,
        overrides: crate::mcp::Servers,
    ) -> Result<(InProc, Ack), Refused> {
        bbt_rt::assert_not_ui_thread();
        if op.v.major != PROTOCOL.major {
            return Err(Refused::Version { server: PROTOCOL });
        }
        let Op::StartSession { cwd, profile, fork } = op.op else {
            return Err(Refused::SessionExists);
        };
        let entry = self.entry(op.session);
        let mut state = entry.lock().await;
        if let Some(loaded) = &state.loaded {
            return if loaded.gateway.session() == op.session && loaded.start == op.op_id {
                Ok((
                    loaded.gateway.clone(),
                    Ack {
                        op_id: op.op_id,
                        duplicate: true,
                    },
                ))
            } else {
                Err(Refused::SessionExists)
            };
        }
        let path = crate::store::log_path(&self.config.sessions, op.session);
        if tokio::fs::try_exists(&path).await.unwrap_or(true) {
            let events = read_log(path).await.map_err(|_| Refused::SessionExists)?;
            if events.first().and_then(|e| e.cause) != Some(op.op_id) {
                return Err(Refused::SessionExists);
            }
            let gateway = self.load(&mut state, op.session, events).await?;
            return Ok((
                gateway,
                Ack {
                    op_id: op.op_id,
                    duplicate: true,
                },
            ));
        }
        let mut config = self.config.clone();
        config.meta.session = op.session;
        config.meta.cwd = cwd;
        config.meta.fork = fork;
        self.profile(&mut config, profile.clone()).await;
        if self.cancel.is_cancelled() {
            return Err(Refused::Closed);
        }
        let agent = self.acp_agent(&mut config, profile.as_deref()).await;
        config.meta.profile = profile;
        let (gateway, task) = self
            .actor(config, op.op_id, None, overrides, agent)
            .await
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Refused::SessionExists
                } else if e.kind() == std::io::ErrorKind::InvalidInput {
                    Refused::BadFork
                } else {
                    Refused::Closed
                }
            })?;
        state.loaded = Some(Loaded {
            gateway: gateway.clone(),
            start: op.op_id,
            task: Some(task),
        });
        Ok((
            gateway,
            Ack {
                op_id: op.op_id,
                duplicate: false,
            },
        ))
    }
    pub async fn open(&self, session: SessionId) -> Result<InProc, Refused> {
        bbt_rt::assert_not_ui_thread();
        let entry = self.entry(session);
        let mut state = entry.lock().await;
        if let Some(loaded) = &state.loaded {
            return if loaded.gateway.session() != session {
                Err(Refused::UnknownSession)
            } else if loaded.gateway.is_closed() {
                Err(Refused::Closed)
            } else {
                Ok(loaded.gateway.clone())
            };
        }
        let path = crate::store::log_path(&self.config.sessions, session);
        match tokio::fs::try_exists(&path).await {
            Ok(false) => return Err(Refused::UnknownSession),
            Err(_) => {
                state.failed_open = true;
                return Err(Refused::Closed);
            }
            Ok(true) => {}
        }
        let events = match read_log(path).await {
            Ok(e) => e,
            Err(_) => {
                state.failed_open = true;
                return Err(Refused::Closed);
            }
        };
        self.load(&mut state, session, events).await
    }
    async fn load(
        &self,
        state: &mut State,
        session: SessionId,
        events: Vec<Envelope>,
    ) -> Result<InProc, Refused> {
        if events
            .last()
            .is_some_and(|e| matches!(e.event, Event::SessionClosed))
        {
            return Err(Refused::Closed);
        }
        let Some(Envelope {
            event: Event::SessionStarted { meta },
            cause,
            ..
        }) = events.first()
        else {
            state.failed_open = true;
            return Err(Refused::Closed);
        };
        if meta.session != session {
            state.failed_open = true;
            return Err(Refused::Closed);
        }
        let mut config = self.config.clone();
        config.meta = meta.clone();
        let start = cause.unwrap_or_default();
        let selected = events
            .iter()
            .rev()
            .find_map(|e| {
                if let Event::ProfileChanged { profile, .. } = &e.event {
                    Some(profile.clone())
                } else {
                    None
                }
            })
            .or_else(|| meta.profile.clone());
        let agent = self.acp_agent(&mut config, selected.as_deref()).await;
        self.profile(&mut config, selected).await;
        if self.cancel.is_cancelled() {
            return Err(Refused::Closed);
        }
        let notice = state
            .failed_open
            .then_some("previous session recovery attempt failed".to_string());
        let (gateway, task) = match self
            .actor(config, start, notice, Default::default(), agent)
            .await
        {
            Ok(value) => value,
            Err(_) => {
                state.failed_open = true;
                return Err(Refused::Closed);
            }
        };
        state.failed_open = false;
        state.loaded = Some(Loaded {
            gateway: gateway.clone(),
            start,
            task: Some(task),
        });
        Ok(gateway)
    }
    /// An `acp-agent` profile selects the ACP agent backend instead of the engine.
    async fn acp_agent(
        &self,
        config: &mut Config,
        name: Option<&str>,
    ) -> Option<(crate::provider::profile::AgentSpec, bool)> {
        let path = Profile::file_in(&config.profiles, name?).ok()?;
        let profile = Profile::load(&path).await.ok()?;
        if !profile.is_acp_agent() {
            return None;
        }
        config.policy = profile.policy();
        Some((
            profile.agent.clone().unwrap_or_default(),
            profile.policy.as_deref() == Some("yolo"),
        ))
    }
    async fn actor(
        &self,
        config: Config,
        start: crate::protocol::OpId,
        notice: Option<String>,
        overrides: crate::mcp::Servers,
        agent: Option<(crate::provider::profile::AgentSpec, bool)>,
    ) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
        match agent {
            #[cfg(feature = "acp-agent")]
            Some((spec, yolo)) => {
                crate::backend::acp_agent::AcpAgentActor::load(config, start, spec, yolo).await
            }
            #[cfg(not(feature = "acp-agent"))]
            Some(_) => {
                SessionActor::load_with_mcp(
                    config,
                    start,
                    Some("ACP agent backend not enabled in this build".into()),
                    overrides,
                    self.launch.clone(),
                )
                .await
            }
            None => {
                SessionActor::load_with_mcp(config, start, notice, overrides, self.launch.clone())
                    .await
            }
        }
    }
    async fn profile(&self, config: &mut Config, name: Option<String>) {
        if name == self.config.meta.profile && self.config.profile_error.is_none() {
            return;
        }
        let Some(name) = name else {
            return;
        };
        let result = async {
            let path = Profile::file_in(&config.profiles, &name)?;
            let profile = Profile::load(&path).await?;
            let key = profile
                .key(&config.redactor, self.cancel.child_token())
                .await?;
            let options = crate::provider::Options {
                retry_deadline: profile.retry_deadline.map(std::time::Duration::from_secs),
                ..Default::default()
            };
            let provider = crate::provider::OpenAiCompat::new(
                &profile.endpoint,
                key,
                config.redactor.clone(),
                options,
            )?;
            Ok::<_, ProviderError>((profile, provider))
        }
        .await;
        match result {
            Ok((profile, provider)) => {
                config.model = profile.model.clone();
                config.policy = profile.policy();
                config.provider = Arc::new(provider);
                config.profile_error = None;
            }
            Err(_) => {
                let message = "profile could not be resolved".to_string();
                config.profile_error = Some(message.clone());
                config.provider = Arc::new(FailedProvider(ProviderError::new(
                    ErrorClass::Config,
                    message,
                )));
            }
        }
    }
    /// Joins this process's actor after an explicit Close. Dropping a subscriber
    /// never invokes this, so detaching preserves the run.
    pub async fn join(&self) -> Result<(), Refused> {
        let entries = self
            .sessions
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let mut tasks = vec![];
        for entry in entries {
            if let Some(task) = entry
                .lock()
                .await
                .loaded
                .as_mut()
                .and_then(|l| l.task.take())
            {
                tasks.push(task);
            }
        }
        let mut result = Ok(());
        for task in tasks {
            if !matches!(task.await, Ok(Ok(()))) {
                result = Err(Refused::Closed);
            }
        }
        result
    }
    /// Stop factories first, then request every actor's independent control lane.
    pub async fn close_all(&self) {
        self.cancel.cancel();
        let entries = self
            .sessions
            .lock()
            .unwrap()
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for entry in entries {
            if let Some(loaded) = &entry.lock().await.loaded {
                let _ = loaded.gateway.send(OpEnvelope {
                    v: PROTOCOL,
                    session: loaded.gateway.session(),
                    op_id: OpId::new(),
                    op: Op::Close,
                });
            }
        }
        let _ = self.join().await;
    }
}
async fn read_log(path: std::path::PathBuf) -> std::io::Result<Vec<Envelope>> {
    crate::runtime::blocking(move || {
        let bytes = std::fs::read(path)?;
        let mut events = vec![];
        for line in bytes.split_inclusive(|b| *b == b'\n') {
            if !line.ends_with(b"\n") {
                break;
            }
            events.push(serde_json::from_slice::<Envelope>(line)?);
        }
        Ok(events)
    })
    .await
}
