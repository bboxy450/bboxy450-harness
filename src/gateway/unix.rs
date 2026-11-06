// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use super::*;
use std::{
    io,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tokio::{
    io::{AsyncBufRead, AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    sync::{Mutex, oneshot},
    task::{JoinHandle, JoinSet},
};
use tokio_util::sync::CancellationToken;
#[derive(Clone, Debug, PartialEq)]
pub enum ClientError {
    Refused(Refused),
    Timeout { op_id: OpId },
    Disconnected,
    Protocol(String),
}
impl std::fmt::Display for ClientError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ClientError {}
async fn frame(reader: &mut (impl AsyncBufRead + Unpin)) -> io::Result<Option<Vec<u8>>> {
    let mut out = vec![];
    loop {
        let buf = reader.fill_buf().await?;
        if buf.is_empty() {
            return if out.is_empty() {
                Ok(None)
            } else {
                Err(io::Error::other("truncated frame"))
            };
        }
        let n = buf
            .iter()
            .position(|b| *b == b'\n')
            .map(|p| p + 1)
            .unwrap_or(buf.len());
        if out.len() + n > MAX_ENVELOPE_BYTES {
            return Err(io::Error::other("frame exceeds 256 KiB"));
        }
        out.extend_from_slice(&buf[..n]);
        reader.consume(n);
        if out.ends_with(b"\n") {
            return Ok(Some(out));
        }
    }
}
async fn write(
    writer: &mut (impl tokio::io::AsyncWrite + Unpin),
    value: &impl Serialize,
) -> io::Result<()> {
    let mut b = serde_json::to_vec(value)?;
    b.push(b'\n');
    if b.len() > MAX_ENVELOPE_BYTES {
        return Err(io::Error::other("frame exceeds 256 KiB"));
    }
    writer.write_all(&b).await
}
pub struct UnixServer {
    pub path: PathBuf,
    gateway: InProc,
    cancel: CancellationToken,
    task: Option<JoinHandle<io::Result<()>>>,
}
#[cfg(test)]
#[derive(Default)]
struct TerminalWriteHold {
    reached: Notify,
    release: Notify,
}
fn lifecycle_terminal(gateway: &InProc) -> bool {
    // An overflowed lifecycle observer may end while admission is sealed but
    // the actor is still joining services. Only publication or an actual actor
    // fault retires that address; sealing alone must resubscribe at the head.
    gateway.hub.state.load().closed || gateway.faulted.load(Ordering::Acquire)
}
impl UnixServer {
    pub async fn bind(path: PathBuf, gateway: InProc) -> io::Result<Self> {
        Self::bind_inner(
            path,
            gateway,
            #[cfg(test)]
            None,
        )
        .await
    }
    async fn bind_inner(
        path: PathBuf,
        gateway: InProc,
        #[cfg(test)] terminal_hold: Option<Arc<TerminalWriteHold>>,
    ) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let listener = UnixListener::bind(&path)?;
        tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await?;
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let own = path.clone();
        let retained_gateway = gateway.clone();
        let hello = gateway.hello();
        let mut lifecycle = gateway
            .subscribe(Subscribe {
                session: gateway.session(),
                after: Some((hello.epoch, hello.head)),
                ephemeral: false,
            })
            .map_err(|_| io::Error::other("session closed"))?;
        lifecycle.set_approver(false);
        let task = bbt_rt::handle().spawn(async move {
            let mut tasks = JoinSet::new();
            let mut unlinked = false;
            let result = loop {
                tokio::select! {
                    biased;
                    _ = stop.cancelled() => break Ok(()),
                    event = lifecycle.recv(), if !unlinked => {
                        if event.as_ref().is_some_and(|e| matches!(e.event, Event::SessionClosed)) || event.is_none() && lifecycle_terminal(&gateway) {
                            let _ = tokio::fs::remove_file(&own).await;
                            unlinked = true;
                            break Ok(());
                        } else if event.is_none() {
                            // A lifecycle observer can lag too. Resubscribe at
                            // the head; losing observer history never unlinks a
                            // live session's address.
                            let hello = gateway.hello();
                            lifecycle = gateway.subscribe(Subscribe {
                                session: gateway.session(), after: Some((hello.epoch, hello.head)), ephemeral: false,
                            }).map_err(|_| io::Error::other("session closed"))?;
                        }
                    }
                    peer = listener.accept() => {
                        let (socket, _) = match peer { Ok(peer) => peer, Err(error) => break Err(error) };
                        if socket.peer_cred().map_or(true, |cred| cred.uid() != unsafe { libc::geteuid() }) { continue; }
                        let g = gateway.clone();
                        let c = stop.child_token();
                        #[cfg(test)]
                        let held = terminal_hold.clone();
                        tasks.spawn_on(async move { let _ = connection(socket, g, c, #[cfg(test)] held).await; }, bbt_rt::handle());
                    }
                    _ = tasks.join_next(), if !tasks.is_empty() => {}
                }
            };
            // Let SessionClosed reach existing subscribers before retiring the
            // address. A stalled reader has a finite one-second drain budget.
            if unlinked {
                let _drained = tokio::time::timeout(Duration::from_secs(1), async {
                    while tasks.join_next().await.is_some() {}
                }).await;
                #[cfg(feature = "test-fixtures")]
                eprintln!("session_close stage=server_drain session={} completed={}", gateway.session(), _drained.is_ok());
            }
            stop.cancel();
            while tasks.join_next().await.is_some() {}
            if !unlinked { let _ = tokio::fs::remove_file(own).await; }
            result
        });
        Ok(Self {
            path,
            gateway: retained_gateway,
            cancel,
            task: Some(task),
        })
    }
    pub async fn retire(mut self) -> io::Result<()> {
        self.task.take().unwrap().await.map_err(io::Error::other)?
    }
    pub async fn close(mut self) -> io::Result<()> {
        // A face may observe the durable close before a Unix writer does.
        // Preserve the existing one-second terminal drain in that order too;
        // admission sealing alone is not evidence of terminal publication.
        let published = self.gateway.hub.state.load().closed;
        #[cfg(feature = "test-fixtures")]
        eprintln!(
            "session_close stage=server_shutdown session={} published={published}",
            self.gateway.session()
        );
        if !published {
            self.cancel.cancel();
        }
        self.task.take().unwrap().await.map_err(io::Error::other)?
    }
}
impl Drop for UnixServer {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
async fn connection(
    socket: UnixStream,
    gateway: InProc,
    cancel: CancellationToken,
    #[cfg(test)] terminal_hold: Option<Arc<TerminalWriteHold>>,
) -> io::Result<()> {
    let (read, mut writer) = socket.into_split();
    let mut reader = BufReader::new(read);
    let first = tokio::select! {_=cancel.cancelled()=>return Ok(()),f=frame(&mut reader)=>f?};
    let first = first.ok_or_else(|| io::Error::other("missing subscribe"))?;
    let value: serde_json::Value = serde_json::from_slice(&first)?;
    // A host-chosen address has no session in its filename. Discovery is
    // subscription-only; every operation still names its admitted session.
    let mut request = value["subscribe"].clone();
    if request.get("session").is_none() {
        request["session"] = serde_json::to_value(gateway.session())?;
    }
    let req: Subscribe = serde_json::from_value(request)?;
    let version: Version = serde_json::from_value(value["v"].clone())?;
    let mut subscription = match gateway.subscribe_version(req, version) {
        Ok(s) => s,
        Err(e) => {
            write(&mut writer, &serde_json::json!({"refused":e})).await?;
            return Ok(());
        }
    };
    // Read-only/legacy clients stay non-approvers. A rich face opts in for the
    // lifetime of this connection; dropping it releases the server-side lease.
    subscription.set_approver(
        value
            .get("approver")
            .and_then(|v| v.as_bool())
            .unwrap_or(false),
    );
    write(&mut writer, subscription.hello()).await?;
    loop {
        let next = frame(&mut reader);
        tokio::pin!(next);
        loop {
            tokio::select! {biased;
              _=cancel.cancelled()=>return Ok(()),
              f=&mut next=>{let Some(f)=f?else{return Ok(());};let op:OpEnvelope=serde_json::from_slice(&f)?;let op_id=op.op_id;
                let response=match gateway.send(op){Ok(ack)=>serde_json::json!({"ack":ack,"op_id":op_id}),Err(refused)=>serde_json::json!({"refused":refused,"op_id":op_id})};
                tokio::select!{_=cancel.cancelled()=>return Ok(()),r=write(&mut writer,&response)=>r?}break;
              },
              e=subscription.recv()=>{let Some(e)=e else{return Ok(());};let done=matches!(e.event,Event::Lagged{..}|Event::Reset{..}|Event::SessionClosed);
                #[cfg(feature = "test-fixtures")]
                if matches!(e.event,Event::SessionClosed) {eprintln!("session_close stage=terminal_dequeued session={} seq={:?}",e.session,e.seq);}
                #[cfg(test)]
                if matches!(e.event,Event::SessionClosed) && let Some(held)=&terminal_hold {
                    held.reached.notify_one();
                    tokio::select!{_=cancel.cancelled()=>return Ok(()),_=held.release.notified()=>{}}
                }
                let bytes=e.wire_bytes()?;tokio::select!{_=cancel.cancelled()=>return Ok(()),r=writer.write_all(&bytes)=>r?};
                #[cfg(feature = "test-fixtures")]
                if matches!(e.event,Event::SessionClosed) {eprintln!("session_close stage=terminal_written session={} seq={:?}",e.session,e.seq);}
                if done{return Ok(());}
              },
            }
        }
    }
}

#[cfg(test)]
mod close_tests {
    use super::*;

    #[test]
    fn ended_lifecycle_requires_terminal_publication_or_actor_fault_not_admission_seal() {
        let (gateway, closed) = super::super::close_fixture();
        gateway.seal();
        assert!(!lifecycle_terminal(&gateway));
        gateway.hub.publish(closed);
        assert!(lifecycle_terminal(&gateway));
        let (faulted, _) = super::super::close_fixture();
        assert!(!lifecycle_terminal(&faulted));
        faulted.mark_faulted();
        assert!(lifecycle_terminal(&faulted));
    }

    #[tokio::test]
    async fn published_close_drains_held_terminal_write_before_server_cancellation() {
        let home = tempfile::tempdir().unwrap();
        let path = home.path().join("held.sock");
        let (gateway, closed) = super::super::close_fixture();
        let hello = gateway.hello();
        let request = Subscribe {
            session: gateway.session(),
            after: Some((hello.epoch, hello.head)),
            ephemeral: false,
        };
        let mut face = gateway.subscribe(request.clone()).unwrap();
        let held = Arc::new(TerminalWriteHold::default());
        let server = UnixServer::bind_inner(path.clone(), gateway.clone(), Some(held.clone()))
            .await
            .unwrap();
        let (client, mut remote) = UnixClient::connect(&path, request).await.unwrap();
        gateway.hub.publish(closed.clone());
        gateway.mark_closed();
        let observed = face.recv().await.unwrap();
        assert!(matches!(observed.event, Event::SessionClosed));
        assert_eq!(observed.seq, closed.seq);
        tokio::time::timeout(Duration::from_secs(5), held.reached.notified())
            .await
            .unwrap();
        // The face has observed the durable close while this actual Unix
        // writer is held before its terminal write. Poll close once in that
        // exact order; it must preserve the server's existing1s drain.
        let cancel = server.cancel.clone();
        let mut closing = Box::pin(server.close());
        assert!(futures_util::poll!(closing.as_mut()).is_pending());
        assert!(
            !cancel.is_cancelled(),
            "published close was eagerly cancelled"
        );
        held.release.notify_one();
        let delivered = tokio::time::timeout(Duration::from_secs(5), remote.recv())
            .await
            .unwrap()
            .unwrap();
        assert_eq!(delivered.session, closed.session);
        assert_eq!(delivered.seq, closed.seq);
        assert!(matches!(delivered.event, Event::SessionClosed));
        assert!(
            tokio::time::timeout(Duration::from_secs(5), remote.recv())
                .await
                .unwrap()
                .is_none(),
            "terminal close must occur exactly once"
        );
        closing.await.unwrap();
        assert!(!path.exists());
        client.close();
    }

    #[tokio::test]
    async fn admission_seal_without_published_close_keeps_immediate_server_cancellation() {
        let home = tempfile::tempdir().unwrap();
        let (gateway, _) = super::super::close_fixture();
        let server = UnixServer::bind(home.path().join("sealed.sock"), gateway.clone())
            .await
            .unwrap();
        gateway.seal();
        let cancel = server.cancel.clone();
        let mut closing = Box::pin(server.close());
        let first = futures_util::poll!(closing.as_mut());
        assert!(cancel.is_cancelled(), "sealing is not durable publication");
        match first {
            std::task::Poll::Ready(result) => result.unwrap(),
            std::task::Poll::Pending => closing.await.unwrap(),
        }
    }
}

type Pending = std::sync::Mutex<BTreeMap<OpId, Vec<oneshot::Sender<Result<Ack, ClientError>>>>>;
struct ClientShared {
    writer: Mutex<tokio::net::unix::OwnedWriteHalf>,
    pending: Pending,
    closed: AtomicBool,
    cancel: CancellationToken,
}
impl Drop for ClientShared {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}
#[derive(Clone)]
pub struct UnixClient {
    shared: Arc<ClientShared>,
}
impl UnixClient {
    /// A stable namespaced key for one bboxy message, independent of retries,
    /// process epochs or the socket address. Retain the original session too.
    pub fn message_op_id(message: &str) -> OpId {
        use sha2::{Digest, Sha256};
        let digest = Sha256::digest(
            [
                b"bboxy450-harness:message:v1\0".as_slice(),
                message.as_bytes(),
            ]
            .concat(),
        );
        OpId(ulid::Ulid::from_bytes(digest[..16].try_into().unwrap()))
    }
    pub async fn connect(path: &Path, sub: Subscribe) -> Result<(Self, Subscription), ClientError> {
        Self::connect_with_approver(path, sub, false).await
    }
    pub async fn connect_with_approver(
        path: &Path,
        sub: Subscribe,
        approver: bool,
    ) -> Result<(Self, Subscription), ClientError> {
        Self::connect_request(path, Some(sub.session), sub.after, sub.ephemeral, approver).await
    }
    /// Learn the session from Hello at an arbitrary host-chosen seat address.
    pub async fn discover(
        path: &Path,
        after: Option<(Epoch, Seq)>,
        ephemeral: bool,
        approver: bool,
    ) -> Result<(Self, Subscription), ClientError> {
        Self::connect_request(path, None, after, ephemeral, approver).await
    }
    async fn connect_request(
        path: &Path,
        session: Option<SessionId>,
        after: Option<(Epoch, Seq)>,
        ephemeral: bool,
        approver: bool,
    ) -> Result<(Self, Subscription), ClientError> {
        bbt_rt::assert_not_ui_thread();
        let mut sub = serde_json::json!({"after": after, "ephemeral": ephemeral});
        if let Some(session) = session {
            sub["session"] = serde_json::to_value(session).unwrap();
        }
        let socket = UnixStream::connect(path)
            .await
            .map_err(|_| ClientError::Disconnected)?;
        let (read, mut writer) = socket.into_split();
        write(
            &mut writer,
            &serde_json::json!({"subscribe":sub,"v":PROTOCOL,"approver":approver}),
        )
        .await
        .map_err(|_| ClientError::Disconnected)?;
        let mut reader = BufReader::new(read);
        let hello = tokio::time::timeout(Duration::from_secs(5), frame(&mut reader))
            .await
            .map_err(|_| ClientError::Protocol("hello timeout".into()))?
            .map_err(|_| ClientError::Disconnected)?
            .ok_or(ClientError::Disconnected)?;
        let value: serde_json::Value = serde_json::from_slice(&hello)
            .map_err(|_| ClientError::Protocol("invalid hello".into()))?;
        if let Some(r) = value.get("refused") {
            return Err(ClientError::Refused(
                serde_json::from_value(r.clone())
                    .map_err(|_| ClientError::Protocol("invalid refusal".into()))?,
            ));
        }
        let hello: Hello = serde_json::from_value(value)
            .map_err(|_| ClientError::Protocol("invalid hello".into()))?;
        if !hello.v.is_readable() {
            return Err(ClientError::Refused(Refused::Version { server: hello.v }));
        }
        let target = Arc::new(Subscriber {
            queue: ArrayQueue::new(4096),
            terminal: ArcSwapOption::empty(),
            notify: Notify::new(),
            waker: ArcSwapOption::empty(),
            last: AtomicU64::new(after.map(|(_, s)| s.0).unwrap_or(0)),
            closed: AtomicBool::new(false),
            ephemeral,
            approver: AtomicBool::new(false),
            drops: AtomicU64::new(0),
        });
        let shared = Arc::new(ClientShared {
            writer: Mutex::new(writer),
            pending: std::sync::Mutex::new(BTreeMap::new()),
            closed: AtomicBool::new(false),
            cancel: CancellationToken::new(),
        });
        let weak = Arc::downgrade(&shared);
        let cancel = shared.cancel.clone();
        let sink = target.clone();
        let meta = hello.meta.clone();
        // One IO owner continuously demultiplexes; subscriber backpressure never
        // prevents a correlated admission response from completing.
        bbt_rt::handle().spawn(async move{let error=loop{let f=tokio::select!{_=cancel.cancelled()=>break ClientError::Disconnected,r=frame(&mut reader)=>r};let bytes=match f{Ok(Some(b))=>b,Ok(None)=>break ClientError::Disconnected,Err(_)=>break ClientError::Protocol("invalid or oversized frame".into())};let value:serde_json::Value=match serde_json::from_slice(&bytes){Ok(v)=>v,Err(_)=>break ClientError::Protocol("invalid JSON".into())};
   if value.get("ack").is_some()||value.get("refused").is_some(){let Some(shared)=weak.upgrade()else{break ClientError::Disconnected;};let op_id:OpId=match serde_json::from_value(value["op_id"].clone()){Ok(id)=>id,Err(_)=>break ClientError::Protocol("uncorrelated response".into())};let answer=if let Some(a)=value.get("ack"){serde_json::from_value(a.clone()).map_err(|_|ClientError::Protocol("invalid ack".into()))}else{match serde_json::from_value(value["refused"].clone()){Ok(r)=>Err(ClientError::Refused(r)),Err(_)=>Err(ClientError::Protocol("invalid refusal".into()))}};if let Some(waiters)=shared.pending.lock().unwrap().remove(&op_id){for waiter in waiters{let _=waiter.send(answer.clone());}}}
   else{match std::str::from_utf8(&bytes).ok().and_then(|line| Envelope::from_line(line).ok()){Some(mut e)=>{e.project_into(&meta);sink.push(e)},None=>break ClientError::Protocol("invalid envelope".into())}}
  };sink.closed.store(true,Ordering::Release);sink.wake();if let Some(s)=weak.upgrade(){s.closed.store(true,Ordering::Release);for (_,waiters) in std::mem::take(&mut *s.pending.lock().unwrap()){for waiter in waiters{let _=waiter.send(Err(error.clone()));}}}});
        let subscription = Subscription {
            hello,
            replay: VecDeque::new(),
            history: im::Vector::new(),
            replay_pos: 0,
            snapshots: VecDeque::new(),
            sub: target,
            cutoff: Seq(0),
            hub: Weak::new(),
        };
        Ok((Self { shared }, subscription))
    }
    pub async fn send(&self, op: OpEnvelope) -> Result<Ack, ClientError> {
        bbt_rt::assert_not_ui_thread();
        let op_id = op.op_id;
        if self.shared.closed.load(Ordering::Acquire) {
            return Err(ClientError::Disconnected);
        }
        let (tx, rx) = oneshot::channel();
        {
            let mut p = self.shared.pending.lock().unwrap();
            p.retain(|_, v| {
                v.retain(|t| !t.is_closed());
                !v.is_empty()
            });
            p.entry(op_id).or_default().push(tx);
        }
        let result = tokio::time::timeout(Duration::from_secs(5), async {
            let mut w = self.shared.writer.lock().await;
            write(&mut *w, &op)
                .await
                .map_err(|_| ClientError::Disconnected)?;
            drop(w);
            rx.await.unwrap_or(Err(ClientError::Disconnected))
        })
        .await;
        result.unwrap_or(Err(ClientError::Timeout { op_id }))
    }
    pub fn close(&self) {
        self.shared.cancel.cancel();
    }
}
