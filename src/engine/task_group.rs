// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! U7 interim: remove when bboxy450-rt exposes scoped task groups.
use tokio::{sync::mpsc, task::JoinSet};
use tokio_util::sync::CancellationToken;
pub struct TaskGroup {
    pub cancel: CancellationToken,
    tasks: JoinSet<()>,
}
impl TaskGroup {
    pub fn new(cancel: CancellationToken) -> Self {
        Self {
            cancel,
            tasks: JoinSet::new(),
        }
    }
    pub fn spawn(&mut self, f: impl std::future::Future<Output = ()> + Send + 'static) {
        self.tasks.spawn_on(f, bbt_rt::handle());
    }
    pub async fn cancel_and_join(&mut self) -> Result<(), tokio::task::JoinError> {
        self.cancel.cancel();
        let mut error = None;
        while let Some(r) = self.tasks.join_next().await {
            if let Err(e) = r {
                error = Some(e);
            }
        }
        if let Some(e) = error { Err(e) } else { Ok(()) }
    }
    pub async fn reap(&mut self) {
        while self.tasks.try_join_next().is_some() {}
    }
}
pub async fn send<T>(tx: &mpsc::Sender<T>, value: T, cancel: &CancellationToken) -> bool {
    tokio::select! {biased;_=cancel.cancelled()=>false,r=tx.send(value)=>r.is_ok()}
}
