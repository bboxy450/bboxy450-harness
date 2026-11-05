// Copyright 2026 Boxy (bboxy450) and contributors.
// SPDX-License-Identifier: LicenseRef-Respectful-Slop-1.0-Draft
// Project: bboxy450-harness — https://github.com/bboxy450/bboxy450-harness
// License: https://github.com/bboxy450/bboxy450-harness/blob/main/LICENSE
// Credit copied/adapted/translated reuse accurately; independent inspiration credit requested.
// Policy: https://github.com/bboxy450/bboxy450-harness/blob/main/ATTRIBUTION.md
//! Best-effort session-start observations, never a live-request or UI operation.
use crate::protocol::{SessionMeta, Vcs};
use std::{path::Path, time::Duration};
use tokio::time::{Instant, timeout_at};
use tokio_util::sync::CancellationToken;

/// One shared deadline for all five sequential queries. The existing owned
/// process runner additionally bounds termination/drain cleanup to 1 second.
pub const CAPTURE_BUDGET: Duration = Duration::from_secs(3);
pub(crate) const MAX_METADATA_BYTES: usize = 16 * 1024;

/// Strip authentication/query data before a known native remote enters raw IR.
/// Other native fields, including unknown Git fields, remain untouched.
pub fn public_remote(remote: &str) -> String {
    if remote.len() >= MAX_METADATA_BYTES {
        return "<redacted>".into();
    }
    if remote.contains("://") {
        let Ok(mut url) = reqwest::Url::parse(remote) else {
            return "<redacted>".into();
        };
        if url.set_username("").is_err() || url.set_password(None).is_err() {
            return "<redacted>".into();
        }
        url.set_query(None);
        url.set_fragment(None);
        return url.to_string();
    }
    let remote = remote.rsplit_once('@').map_or(remote, |(_, rest)| rest);
    remote
        .split(['?', '#'])
        .next()
        .unwrap_or_default()
        .to_owned()
}

/// memory-graph §1.1: host/owner/name, without scheme, authentication or .git.
/// Unsupported/local-path origins stay unknown rather than inventing a host.
pub fn normalize_repo_key(remote: &str) -> Option<String> {
    if remote.is_empty()
        || remote.len() > MAX_METADATA_BYTES
        || remote.chars().any(char::is_control)
    {
        return None;
    }
    let safe = public_remote(remote);
    let url = if safe.contains("://") {
        reqwest::Url::parse(&safe).ok()?
    } else {
        let (host, path) = safe.split_once(':')?;
        reqwest::Url::parse(&format!("ssh://{host}/{}", path.trim_start_matches('/'))).ok()?
    };
    if !matches!(url.scheme(), "http" | "https" | "ssh" | "git") {
        return None;
    }
    let mut host = url.host_str()?.to_ascii_lowercase();
    if let Some(port) = url.port() {
        host = format!("{host}:{port}");
    }
    let path = url.path().trim_matches('/');
    let path = path.strip_suffix(".git").unwrap_or(path);
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() < 2
        || parts
            .iter()
            .any(|p| p.is_empty() || *p == "." || *p == "..")
    {
        return None;
    }
    Some(format!("{host}/{path}"))
}

struct Query {
    code: i32,
    bytes: zeroize::Zeroizing<Vec<u8>>,
}
impl Query {
    fn text(&self) -> Option<&str> {
        (self.code == 0).then_some(())?;
        let text = std::str::from_utf8(&self.bytes)
            .ok()?
            .trim_end_matches(['\r', '\n']);
        (!text.is_empty() && !text.contains(['\n', '\r', '\0'])).then_some(text)
    }
}
async fn git(cwd: &Path, args: &[&str], deadline: Instant) -> Option<Query> {
    let remaining = deadline.checked_duration_since(Instant::now())?;
    // No shell, pager, credential helper, fsmonitor hook, transport or operator
    // configuration. Repository-local metadata is the only intended input.
    let mut command = Vec::new();
    // Git tracing can write parent environment secrets directly to a file or
    // socket, bypassing stdout capture. Clear its whole inherited namespace.
    for name in std::env::vars_os()
        .filter_map(|(name, _)| name.into_string().ok())
        .filter(|name| name.starts_with("GIT_"))
    {
        command.extend(["-u".into(), name]);
    }
    command.extend(
        [
            "GIT_CONFIG_NOSYSTEM=1",
            "GIT_CONFIG_GLOBAL=/dev/null",
            "GIT_TERMINAL_PROMPT=0",
            "GIT_ASKPASS=/usr/bin/false",
            "SSH_ASKPASS=/usr/bin/false",
            "git",
            "--no-pager",
            "--no-optional-locks",
            "-c",
            "core.fsmonitor=false",
            "-c",
            "core.hooksPath=/dev/null",
            "-c",
            "credential.helper=",
            "-c",
            "protocol.allow=never",
        ]
        .map(String::from),
    );
    command.extend(args.iter().map(|s| (*s).to_owned()));
    let output = crate::process::capture_stdout(
        crate::process::Request {
            program: "/usr/bin/env".into(),
            args: command,
            stdin: None,
            cwd: cwd.into(),
            timeout: remaining,
            spill_dir: Default::default(),
            cap: MAX_METADATA_BYTES,
            redactor: Default::default(),
        },
        CancellationToken::new(),
    )
    .await
    .ok()?;
    if output.cancelled || output.bytes.len() >= MAX_METADATA_BYTES {
        return None;
    }
    Some(Query {
        code: output.code?,
        bytes: zeroize::Zeroizing::new(output.bytes),
    })
}
async fn locate(cwd: &Path, deadline: Instant) -> Option<Option<String>> {
    let root = git(cwd, &["rev-parse", "--show-toplevel"], deadline).await?;
    let root = root.text()?.to_owned();
    let origin = git(cwd, &["config", "--get", "remote.origin.url"], deadline).await;
    match origin {
        Some(origin) if origin.code == 0 => Some(origin.text().and_then(normalize_repo_key)),
        Some(origin) if origin.code == 1 => {
            let canonical = timeout_at(
                deadline,
                crate::runtime::blocking(move || {
                    crate::runtime::assert_off_rt_worker();
                    std::fs::canonicalize(root)
                }),
            )
            .await
            .ok()?
            .ok()?;
            Some(Some(format!("path:{}", canonical.to_str()?)))
        }
        _ => Some(None),
    }
}
/// Resolve only repository identity, using the same normalization as import.
pub async fn repo_key(cwd: &Path) -> Option<String> {
    locate(cwd, Instant::now() + CAPTURE_BUDGET).await.flatten()
}
async fn capture_until(cwd: &Path, deadline: Instant) -> Option<Vcs> {
    let repo = locate(cwd, deadline).await?;
    let branch = git(cwd, &["rev-parse", "--abbrev-ref", "HEAD"], deadline)
        .await
        .and_then(|q| q.text().map(str::to_owned));
    let commit = git(cwd, &["rev-parse", "--verify", "HEAD"], deadline)
        .await
        .and_then(|q| q.text().map(str::to_owned));
    let dirty = git(
        cwd,
        &[
            "status",
            "--porcelain=v1",
            "--untracked-files=normal",
            // Override repository/.gitmodules ignore policy: changed gitlinks
            // and submodule worktrees are part of this checkout's dirty state.
            "--ignore-submodules=none",
        ],
        deadline,
    )
    .await
    .filter(|q| q.code == 0)
    .map(|q| !q.bytes.is_empty());
    let vcs = Vcs {
        repo,
        branch,
        commit,
        dirty,
    };
    (vcs != Vcs::default()).then_some(vcs)
}
/// Failed or timed-out queries remain absent; errors never echo remote output.
pub async fn capture(cwd: &Path) -> Option<Vcs> {
    capture_until(cwd, Instant::now() + CAPTURE_BUDGET).await
}
/// Existing logs keep their original header and never run a new Git probe.
pub(crate) async fn capture_new_session(meta: &mut SessionMeta, sessions: &Path) {
    if meta.vcs.is_some() {
        return;
    }
    let deadline = Instant::now() + CAPTURE_BUDGET;
    if matches!(
        timeout_at(
            deadline,
            tokio::fs::try_exists(crate::store::log_path(sessions, meta.session))
        )
        .await,
        Ok(Ok(false))
    ) {
        meta.vcs = capture_until(&meta.cwd, deadline).await;
    }
}
