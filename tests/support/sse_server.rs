// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};
pub struct Server {
    pub endpoint: String,
    pub attempts: Arc<AtomicUsize>,
    #[allow(dead_code)]
    pub requests: Arc<Mutex<Vec<Vec<u8>>>>,
    task: tokio::task::JoinHandle<()>,
}
impl Drop for Server {
    fn drop(&mut self) {
        self.task.abort();
    }
}
#[derive(Clone)]
pub struct Reply {
    pub status: u16,
    pub body: Vec<u8>,
    pub retry_after: Option<u64>,
    pub hold: Duration,
    pub truncated: bool,
}
impl Server {
    pub async fn start(replies: Vec<Reply>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = format!("http://{}/v1", listener.local_addr().unwrap());
        let attempts = Arc::new(AtomicUsize::new(0));
        let n = attempts.clone();
        let requests = Arc::new(Mutex::new(Vec::new()));
        let captured = requests.clone();
        let task = tokio::spawn(async move {
            while let Ok((mut socket, _)) = listener.accept().await {
                let index = n.fetch_add(1, Ordering::SeqCst);
                let Some(reply) = replies.get(index).cloned() else {
                    break;
                };
                let mut request = vec![];
                let mut buf = [0; 4096];
                loop {
                    let c = socket.read(&mut buf).await.unwrap_or(0);
                    if c == 0 {
                        break;
                    }
                    request.extend_from_slice(&buf[..c]);
                    if let Some(end) = request.windows(4).position(|b| b == b"\r\n\r\n") {
                        let headers = String::from_utf8_lossy(&request[..end]);
                        let length = headers
                            .lines()
                            .find_map(|line| {
                                let (name, value) = line.split_once(':')?;
                                name.eq_ignore_ascii_case("content-length")
                                    .then(|| value.trim().parse::<usize>().ok())
                                    .flatten()
                            })
                            .unwrap_or(0);
                        if request.len() >= end + 4 + length {
                            break;
                        }
                    }
                }
                captured.lock().unwrap().push(request);
                let mut body = reply.body;
                let mut cuts = vec![];
                while let Some(at) = body.windows(7).position(|b| b == b"#split\n") {
                    body.drain(at..at + 7);
                    cuts.push(at);
                }
                cuts.push(body.len());
                let length = body.len() + if reply.truncated { 1024 } else { 0 };
                let headers = format!(
                    "HTTP/1.1 {} fixture\r\nContent-Type: text/event-stream\r\nContent-Length: {length}\r\nConnection: close\r\n{}\r\n",
                    reply.status,
                    reply
                        .retry_after
                        .map(|n| format!("Retry-After: {n}\r\n"))
                        .unwrap_or_default()
                );
                if socket.write_all(headers.as_bytes()).await.is_err() {
                    continue;
                }
                let mut start = 0;
                for end in cuts {
                    for chunk in body[start..end].chunks(7) {
                        if socket.write_all(chunk).await.is_err() {
                            break;
                        }
                    }
                    let _ = socket.flush().await;
                    tokio::task::yield_now().await;
                    start = end;
                }
                let _ = socket.flush().await;
                tokio::time::sleep(reply.hold).await;
            }
        });
        Self {
            endpoint,
            attempts,
            requests,
            task,
        }
    }
}
pub fn ok(body: Vec<u8>) -> Reply {
    Reply {
        status: 200,
        body,
        retry_after: None,
        hold: Duration::ZERO,
        truncated: false,
    }
}
