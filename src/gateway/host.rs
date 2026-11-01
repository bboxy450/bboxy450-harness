// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Async loading boundary. No disk read or recovery occurs in Gateway::subscribe.
use super::*;
use crate::{
    engine::{Config, SessionActor},
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
    state: Mutex<State>,
}
impl Host {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            state: Mutex::new(State {
                loaded: None,
                failed_open: false,
            }),
        }
    }
    pub async fn create(&self, op: OpEnvelope) -> Result<(InProc, Ack), Refused> {
        bbt_rt::assert_not_ui_thread();
        if op.v.major != PROTOCOL.major {
            return Err(Refused::Version { server: PROTOCOL });
        }
        let Op::StartSession { cwd, profile, fork } = op.op else {
            return Err(Refused::SessionExists);
        };
        let mut state = self.state.lock().await;
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
        config.meta.profile = profile;
        let (gateway, task) = SessionActor::load(config, op.op_id, None)
            .await
            .map_err(|_| Refused::Closed)?;
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
        let mut state = self.state.lock().await;
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
        self.profile(&mut config, selected).await;
        let notice = state
            .failed_open
            .then_some("previous session recovery attempt failed".to_string());
        let (gateway, task) = match SessionActor::load(config, start, notice).await {
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
                .key(&config.redactor, CancellationToken::new())
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
        let task = {
            let mut state = self.state.lock().await;
            state.loaded.as_mut().and_then(|l| l.task.take())
        };
        if let Some(task) = task {
            task.await
                .map_err(|_| Refused::Closed)?
                .map_err(|_| Refused::Closed)?;
        }
        Ok(())
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
