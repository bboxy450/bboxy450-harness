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
    cancel: CancellationToken,
    task: Option<JoinHandle<io::Result<()>>>,
}
impl UnixServer {
    pub async fn bind(path: PathBuf, gateway: InProc) -> io::Result<Self> {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let listener = UnixListener::bind(&path)?;
        tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await?;
        let cancel = CancellationToken::new();
        let stop = cancel.clone();
        let own = path.clone();
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
                        if gateway.is_closed() || event.as_ref().is_some_and(|e| matches!(e.event, Event::SessionClosed)) {
                            let _ = tokio::fs::remove_file(&own).await;
                            unlinked = true;
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
                        tasks.spawn_on(async move { let _ = connection(socket, g, c).await; }, bbt_rt::handle());
                    }
                    _ = tasks.join_next(), if !tasks.is_empty() => {}
                }
            };
            stop.cancel();
            while tasks.join_next().await.is_some() {}
            if !unlinked { let _ = tokio::fs::remove_file(own).await; }
            result
        });
        Ok(Self {
            path,
            cancel,
            task: Some(task),
        })
    }
    pub async fn close(mut self) -> io::Result<()> {
        self.cancel.cancel();
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
) -> io::Result<()> {
    let (read, mut writer) = socket.into_split();
    let mut reader = BufReader::new(read);
    let first = tokio::select! {_=cancel.cancelled()=>return Ok(()),f=frame(&mut reader)=>f?};
    let first = first.ok_or_else(|| io::Error::other("missing subscribe"))?;
    let value: serde_json::Value = serde_json::from_slice(&first)?;
    let req: Subscribe = serde_json::from_value(value["subscribe"].clone())?;
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
                let bytes=e.wire_bytes()?;tokio::select!{_=cancel.cancelled()=>return Ok(()),r=writer.write_all(&bytes)=>r?};if done{return Ok(());}
              },
            }
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
    pub async fn connect(path: &Path, sub: Subscribe) -> Result<(Self, Subscription), ClientError> {
        Self::connect_with_approver(path, sub, false).await
    }
    pub async fn connect_with_approver(
        path: &Path,
        sub: Subscribe,
        approver: bool,
    ) -> Result<(Self, Subscription), ClientError> {
        bbt_rt::assert_not_ui_thread();
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
            last: AtomicU64::new(sub.after.map(|(_, s)| s.0).unwrap_or(0)),
            closed: AtomicBool::new(false),
            ephemeral: sub.ephemeral,
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
