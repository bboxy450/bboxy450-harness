// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Session-private stdio relay. Only opaque socket paths cross the ACP wire.
//! macOS ships /usr/bin/nc; it is a byte transport, not a second agent/tool loop.
use super::*;
use std::{
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
};
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::{UnixListener, UnixStream},
    time::Instant,
};

pub(crate) struct Relay {
    pub servers: Vec<Value>,
    directory: PathBuf,
}
impl Relay {
    pub async fn start(
        configs: SpawnServers,
        cwd: &Path,
        redactor: Set,
        output: mpsc::Sender<Internal>,
        services: &mut TaskGroup,
    ) -> std::io::Result<Self> {
        // Short Darwin socket names; no HOME/seat path, credentials or config in argv.
        let directory = PathBuf::from("/tmp").join(format!("bh-mcp-{}", ulid::Ulid::new()));
        tokio::fs::DirBuilder::new()
            .mode(0o700)
            .create(&directory)
            .await?;
        let mut relay = Self {
            servers: vec![],
            directory,
        };
        for (i, (name, config)) in configs.into_iter().enumerate() {
            if i >= 16 {
                notice(
                    &output,
                    &services.cancel,
                    Level::Warn,
                    format!("mcp {name}: server limit 16"),
                )
                .await;
                continue;
            }
            if validate_environment(config.origin, &config.config.env).is_err() {
                notice(
                    &output,
                    &services.cancel,
                    Level::Warn,
                    format!("mcp {name}: configuration error: reserved environment reference"),
                )
                .await;
                continue;
            }
            let path = relay.directory.join(i.to_string());
            let listener = UnixListener::bind(&path)?;
            tokio::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).await?;
            relay
                .servers
                .push(json!({"name":name,"command":"/usr/bin/nc","args":["-U",path],"env":[]}));
            let cancel = services.cancel.child_token();
            let output = output.clone();
            let redactor = redactor.clone();
            let cwd = cwd.to_owned();
            services.spawn(async move {
                loop {
                    let connection = tokio::select! { biased;
                        _ = cancel.cancelled() => break,
                        connection = listener.accept() => connection,
                    };
                    let Ok((stream, _)) = connection else { break };
                    // The directory/socket ACL restrict this endpoint to its owning uid.
                    if !stream.peer_cred().is_ok_and(|peer| peer.uid() == unsafe { libc::geteuid() }) { continue; }
                    let result = serve(stream, &config, &cwd, &redactor, &cancel).await;
                    if cancel.is_cancelled() { break; }
                    if result.is_err() {
                        notice(&output, &cancel, Level::Warn, format!("mcp {name}: relay unavailable or request deadline exceeded ({} origin)",config.origin.label())).await;
                    }
                }
            });
        }
        Ok(relay)
    }
    pub fn remove(&self) {
        let _ = std::fs::remove_dir_all(&self.directory);
    }
}
impl Drop for Relay {
    fn drop(&mut self) {
        self.remove();
    }
}

async fn serve(
    stream: UnixStream,
    selected: &SpawnConfig,
    cwd: &Path,
    redactor: &Set,
    cancel: &CancellationToken,
) -> Result<(), ()> {
    let mut command = tokio::select! { biased;
        _ = cancel.cancelled() => return Ok(()),
        r = tokio::time::timeout(Duration::from_secs(10), server_command(&selected.config, selected.origin, redactor, cancel)) => r.map_err(|_| ())?.map_err(|_| ())?,
    };
    command
        .current_dir(cwd)
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::null());
    let mut child = crate::process::OwnedChild::spawn(command).map_err(|_| ())?;
    let mut stdin = child.stdin().ok_or(())?;
    let mut stdout = BufReader::new(child.stdout().ok_or(())?);
    let mut server_line = Vec::new();
    let (read, mut write) = stream.into_split();
    let mut read = BufReader::new(read);
    let mut client_line = Vec::new();
    // Initialization and each request have a deadline; a stalled server cannot
    // leave the downstream agent waiting indefinitely. No raw error/secret is logged.
    let mut pending = BTreeMap::<String, Instant>::new();
    let mut startup = Some(Instant::now() + Duration::from_secs(10));
    loop {
        let deadline = pending.values().copied().chain(startup).min();
        tokio::select! { biased;
            _ = cancel.cancelled() => return Ok(()),
            _ = async { match deadline { Some(d) => tokio::time::sleep_until(d).await, None => std::future::pending().await } } => return Err(()),
            line = next_line(&mut read, &mut client_line) => {
                let Some(line) = line.map_err(|_| ())? else { return Ok(()) };
                let value = line_value(&line, redactor)?;
                if let Some(id) = value.get("id") && value.get("method").is_some() {
                        if pending.len() >= 128 { return Err(()) }
                        let timeout = if value["method"] == "initialize" {10} else {selected.config.timeout_s.max(1)};
                        pending.insert(id.to_string(), Instant::now() + Duration::from_secs(timeout));
                }
                let mut bytes = serde_json::to_vec(&value).map_err(|_| ())?; bytes.push(b'\n');
                tokio::select! { biased; _ = cancel.cancelled() => return Ok(()), r = tokio::time::timeout(Duration::from_secs(10), stdin.write_all(&bytes)) => { r.map_err(|_| ())?.map_err(|_| ())?; } }
            }
            line = next_line(&mut stdout, &mut server_line) => {
                let Some(line) = line.map_err(|_| ())? else { return Err(()) };
                let value = line_value(&line, redactor)?;
                if value.get("method").is_none() && let Some(id) = value.get("id") {
                    pending.remove(&id.to_string()); startup = None;
                }
                let mut bytes = serde_json::to_vec(&value).map_err(|_| ())?; bytes.push(b'\n');
                tokio::select! { biased; _ = cancel.cancelled() => return Ok(()), r = tokio::time::timeout(Duration::from_secs(10), write.write_all(&bytes)) => { r.map_err(|_| ())?.map_err(|_| ())?; } }
            }
        }
    }
}
fn line_value(line: &str, redactor: &Set) -> Result<Value, ()> {
    if line.len() > 8 * 1024 * 1024 {
        return Err(());
    }
    serde_json::from_str(line)
        .map(|v| redactor.value(v))
        .map_err(|_| ())
}

// read_until is cancellation safe when its buffer survives the select; cap before
// allocation, including a peer that never writes a newline.
async fn next_line<R: tokio::io::AsyncBufRead + Unpin>(
    reader: &mut R,
    buffer: &mut Vec<u8>,
) -> std::io::Result<Option<String>> {
    const MAX: usize = 8 * 1024 * 1024;
    let count = reader
        .take((MAX + 1 - buffer.len()) as u64)
        .read_until(b'\n', buffer)
        .await?;
    if buffer.len() > MAX {
        return Err(std::io::Error::other("MCP line too long"));
    }
    if count == 0 && buffer.is_empty() {
        return Ok(None);
    }
    String::from_utf8(std::mem::take(buffer))
        .map(Some)
        .map_err(std::io::Error::other)
}
