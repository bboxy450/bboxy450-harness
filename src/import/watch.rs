// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md

//! A live session over a native log: tail a Claude Code or Codex session file, project
//! each new record into our events, and serve them through the same Gateway, so every
//! face renders another harness's running session. It is read-only: a turn is refused
//! with a pointer to `/profile`, which forks the session onto our engine or an ACP agent
//! (with the conversation handed over) to continue it.
use super::{Native, Projector, spill};
use crate::{
    engine::Config,
    gateway::{Admitted, Hub, InProc},
    protocol::*,
    store::Store,
};
use serde_json::Value;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tokio::{io::AsyncReadExt, sync::mpsc};

pub const POLL: Duration = Duration::from_millis(250);

pub(crate) async fn load(
    mut config: Config,
    start: OpId,
    native: Option<Native>,
    path: PathBuf,
) -> std::io::Result<(InProc, tokio::task::JoinHandle<std::io::Result<()>>)> {
    let mut store = Store::open(
        config.sessions.clone(),
        config.meta.clone(),
        Some(start),
        config.redactor.clone(),
    )
    .await?;
    if let Some(Envelope {
        event: Event::SessionStarted { meta },
        ..
    }) = store.events.first()
    {
        config.meta = meta.clone();
    }
    let hub = Arc::new(Hub::new(
        config.meta.clone(),
        store.epoch,
        &store.events,
        config.redactor.clone(),
    ));
    let (work_tx, mut work) = mpsc::channel::<Admitted>(64);
    let (control_tx, mut control) = mpsc::channel::<Admitted>(256);
    let gateway = InProc::new(hub, work_tx, control_tx, start);
    let fresh = store.fresh;
    let g = gateway.clone();
    let task = bbt_rt::handle().spawn(async move {

        let label = native.map(Native::label).unwrap_or("native");
        if fresh {
            let e = store
                .append(Event::ProfileChanged { profile: format!("watch:{label}"), model: path.display().to_string() }, Some(start))
                .await?;
            g.hub.publish(e);
        }
        let mut projector: Option<Projector> = native.map(Projector::new);
        let mut offset = 0u64;
        let mut partial = String::new();
        let mut tick = tokio::time::interval(POLL);
        let result: std::io::Result<()> = async {
            loop {
                tokio::select! {biased;
                    Some(op) = recv_any(&mut control, &mut work) => {
                        if matches!(op.env.op, Op::Close) && let Some(p) = projector.as_mut() {
                            // the native turn may still run; our copy of it ends here
                            for event in p.finish(StopReason::Interrupted) {
                                publish(&mut store, &g.hub, event).await?;
                            }
                        }
                        if handle(op, &mut store, &g.hub).await? { break; }
                    }
                    _ = tick.tick() => {
                        // read whatever was appended since the last poll; only whole lines count
                        let Ok(mut file) = tokio::fs::File::open(&path).await else { continue };
                        let len = file.metadata().await.map(|m| m.len()).unwrap_or(0);
                        if len < offset { offset = 0; partial.clear(); } // truncated or replaced
                        if len == offset { continue; }
                        use tokio::io::AsyncSeekExt;
                        file.seek(std::io::SeekFrom::Start(offset)).await?;
                        let mut buf = Vec::new();
                        file.take(len - offset).read_to_end(&mut buf).await?;
                        offset = len;
                        partial.push_str(&String::from_utf8_lossy(&buf));
                        let mut lines: Vec<String> = partial.split('\n').map(str::to_owned).collect();
                        partial = lines.pop().unwrap_or_default();
                        for line in lines {
                            let Ok(record) = serde_json::from_str::<Value>(&line) else { continue };
                            let p = projector.get_or_insert_with(|| Projector::new(Native::sniff(&record).unwrap_or(Native::ClaudeCode)));
                            let events = p.push(&record);
                            let at = p.ts_ms.unwrap_or_else(crate::store::now_ms);
                            for event in events {
                                let e = spill::append(&mut store, event, at).await?;
                                g.hub.publish(e);
                            }
                        }
                        g.hub.flush_deltas();
                    }
                }
            }
            Ok(())
        }
        .await;
        g.mark_closed();
        result
    });
    Ok((gateway, task))
}

/// Ops on a watched session: Close ends the watch; a turn is refused with the way forward.
async fn handle(op: Admitted, store: &mut Store, hub: &Hub) -> std::io::Result<bool> {
    let id = op.env.op_id;
    let (event, closing) = match op.env.op {
        Op::Close => (Event::SessionClosed, true),
        _ => (
            Event::OpRejected {
                op: id,
                reason: Rejection::InvalidInput {
                    detail: "a watched session is read-only: /profile NAME continues it on our engine or an ACP agent".into(),
                },
            },
            false,
        ),
    };
    let e = if event.is_durable() {
        store.append(event, Some(id)).await?
    } else {
        store.ephemeral(event)
    };
    hub.publish(e);
    Ok(closing)
}

async fn recv_any(
    control: &mut mpsc::Receiver<Admitted>,
    work: &mut mpsc::Receiver<Admitted>,
) -> Option<Admitted> {
    tokio::select! {biased;
        Some(op) = control.recv() => Some(op),
        Some(op) = work.recv() => Some(op),
        else => None,
    }
}

async fn publish(store: &mut Store, hub: &Hub, event: Event) -> std::io::Result<()> {
    let e = spill::append(store, event, crate::store::now_ms()).await?;
    hub.publish(e);
    Ok(())
}
